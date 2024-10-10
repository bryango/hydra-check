import json
import logging
from sys import exit as sysexit
import sys
from typing import Callable, Dict, Iterator, Union

import requests
from bs4 import BeautifulSoup, element
from colorama import Fore, Style

from hydra_check.arguments import process_args

# TODO: use TypedDict
EvalStatus = Dict[str, Union[str, bool, int]]


def get_evals(ident: str) -> str:
    return f"https://hydra.nixos.org/jobset/{ident}/evals"


def get_one_eval(ident: str) -> str:
    if not ident.isnumeric:
        logging.error(f"evaluation should be identified by a number, not {ident}")
    return f"https://hydra.nixos.org/eval/{ident}"


def fetch_data(ident: str, get_url: Callable[[str], str]) -> str:
    # https://hydra.nixos.org/jobset/nixpkgs/trunk/evals
    # https://hydra.nixos.org/eval/1809297
    url = get_url(ident)
    resp = requests.get(url, timeout=20)
    if resp.status_code == 404:
        print(f"package {ident} not found at url {url}")
        sysexit(1)
    return resp.text


def parse_jobset_html(data: str) -> Iterator[EvalStatus]:
    doc = BeautifulSoup(data, features="html.parser")
    if not doc.find("tbody"):
        # Either the package was not evaluated (due to being unfree)
        # or the package does not exist
        alert_text = ""
        if result := doc.find("div", {"class": "alert"}):
            alert_text = result.text.replace("\n", " ")
        else:
            alert_text = "Unknown Hydra Error, check the package with --url to find out what went wrong"

        yield {"icon": "⚠", "status": alert_text}
        return

    if tbody := doc.find("tbody"):
        if isinstance(tbody, element.Tag):
            for row in tbody.find_all("tr"):
                eval_id, timestamp, input_changes, built, failed, pending, delta = (
                    row.find_all("td")
                )

                url = eval_id.find("a")["href"]
                eval_id = eval_id.text

                date = timestamp.find("time")["datetime"]
                relative = timestamp.find("time").text
                timestamp = timestamp.find("time")["data-timestamp"]

                status = input_changes.find("span").string
                short_rev = input_changes.find("tt").string
                input_changes.find("span").clear()
                input_changes = " ".join(input_changes.text.strip().split())

                built = built.text.strip()
                failed = failed.text.strip()
                pending = pending.text.strip()
                delta = delta.text.strip()

                finished = pending == ""
                icon = "✔" if finished else "⧖"
                yield {
                    "icon": icon,
                    "finished": finished,
                    "id": int(eval_id),
                    "url": url,
                    "datetime": date,
                    "relative": relative,
                    "timestamp": int(timestamp),
                    "status": status,
                    "short_rev": short_rev,
                    "input_changes": input_changes,
                    "built": int(built) if built else 0,
                    "failed": int(failed) if failed else 0,
                    "pending": int(pending) if pending else 0,
                    "delta": delta,  # str with a + or - sign
                }


def print_jobset_eval(evaluation: EvalStatus) -> None:
    match evaluation["icon"]:
        case "✖":
            print(Fore.RED, end="")
        case "⚠" | "⧖":
            print(Fore.YELLOW, end="")
        case "✔":
            print(Fore.GREEN, end="")
    if "built" in evaluation:
        extra = f" ({evaluation['status']})"
        print(
            f"{evaluation['icon']}{extra} {evaluation['delta']} from "
            f"{str(evaluation['timestamp']).split('T', maxsplit=1)[0]} - {evaluation['url']}",
        )
    else:
        print(f"{evaluation['icon']} {evaluation['status']}")
