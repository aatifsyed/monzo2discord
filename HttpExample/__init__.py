import json
import logging
import time
from typing import Any, Optional

import azure.functions as func
import requests


def post_transaction_to_discord(url: str, transaction_json: dict):
    amount = int(transaction_json["data"]["amount"])
    currency = transaction_json["data"]["currency"]
    merchant = transaction_json["data"]["merchant"]["name"]
    message = f"""💸 New transaction:
    💷 {amount/100}{currency} ({"📉" if amount < 0 else "📈"})
    📌 {merchant}"""

    response = requests.post(url=url, json={"content": message})

    logging.info(
        f""""Discord:,
    {response.status_code},
    {response.reason},
    {response.content}"""
    )


def post_monzo_balance_to_discord(
    url: str,
    statein: func.InputStream,
    stateout: func.Out[func.InputStream],
):
    pass


def main(
    request: func.HttpRequest,
    discordfile: func.InputStream,
    monzofile: func.InputStream,
    statein: func.InputStream,
    stateout: func.Out[func.InputStream],
) -> func.HttpResponse:
    logging.info("Received a request")

    logging.info(transaction_json := request.get_json())
    logging.info(monzofile := json.load(monzofile))
    logging.info(discordfile := json.load(discordfile))

    # post_transaction_to_discord(
    #     url=discordfile["url"], transaction_json=transaction_json
    # )

    return func.HttpResponse(
        status_code=200,
    )
