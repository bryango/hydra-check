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
                eval_id, timestamp, input_changes, succeeded, failed, queued, delta = (
                    row.find_all("td")
                )

                url = eval_id.find("a")["href"]
                eval_id = eval_id.text

                date = timestamp.find("time")["datetime"]
                relative = timestamp.find("time").text
                timestamp = timestamp.find("time")["data-timestamp"]

                status = getattr(input_changes.find("span"), "string", "")
                short_rev = input_changes.find("tt").string
                getattr(input_changes.find("span"), "clear", lambda: ())()
                input_changes = " ".join(input_changes.text.strip().split())

                succeeded = succeeded.text.strip()
                failed = failed.text.strip()
                queued = queued.text.strip()
                delta = delta.text.strip()

                finished = queued == ""
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
                    "succeeded": int(succeeded) if succeeded else 0,
                    "failed": int(failed) if failed else 0,
                    "queued": int(queued) if queued else 0,
                    "delta": delta,  # str with a + or - sign
                }


def print_jobset_eval(e: EvalStatus) -> None:
    match e["icon"]:
        case "✖":
            entry_color = Fore.RED
        case "⚠" | "⧖":
            entry_color = Fore.YELLOW
        case "✔":
            entry_color = Fore.GREEN
    print(entry_color, end="")
    if "id" in e:
        queued = f"? {e['queued']}"
        if e["queued"] != 0:
            queued = f"{Fore.YELLOW}{queued}{entry_color}"

        delta = f"Δ {e['delta']}"
        if isinstance(e["delta"], str) and e["delta"].startswith("-"):
            delta = f"{Fore.RED}{delta}{entry_color}"
        print(
            f"{e['icon']} {e['input_changes']} from {e['relative']:10}"
            f"  ✔ {e['succeeded']} ✖ {e['failed']} {queued} {delta}"
            f"  {e['url']}",
        )
    else:
        print(f"{e['icon']} {e['status']}")
