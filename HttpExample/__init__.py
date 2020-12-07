import json
import logging
from os import stat
import time
from types import resolve_bases
from typing import Any, Optional

import azure.functions as func
import requests


def post_message_to_discord(url: str, message: str):
    response = requests.post(url=url, json={"content": message})

    logging.info(
        f""""Discord:,
    {response.status_code},
    {response.reason},
    {response.content}"""
    )


def post_transaction_to_discord(url: str, transaction_json: dict):
    amount = int(transaction_json["data"]["amount"])
    currency = transaction_json["data"]["currency"]
    merchant = transaction_json["data"]["merchant"]["name"]
    message = f"""💸 New transaction:
    💷 {amount/100}{currency} ({"📉" if amount < 0 else "📈"})
    📌 {merchant}"""

    post_message_to_discord(url=url, message=message)


# fragile
def refresh_monzo_tokens(
    monzo: dict, statein: func.InputStream, stateout: func.Out[func.InputStream]
):
    old_state = json.load(statein)
    logging.info(old_state)
    old_access_token = old_state["access_token"]
    old_refresh_token = old_state["refresh_token"]

    response = requests.post(
        url="https://api.monzo.com/oauth2/token",
        data={
            "grant_type": "refresh_token",
            "client_id": monzo["client_id"],
            "client_secret": monzo["client_secret"],
            "refresh_token": old_refresh_token,
        },
    )

    logging.info(
        f""""Monzo refresh:,
    {response.status_code},
    {response.reason},
    {response.content}"""
    )
    response = response.json()
    new_state = {
        "access_token": response["access_token"],
        "refresh_token": response["refresh_tokens"],
    }
    stateout.set(json.dumps(new_state))
    return response["access_token"]


def monzo_get_balance(monzo: dict, access_token):
    response = requests.post(
        url="https://api.monzo.com/balance",
        headers={"Authorization": f"Bearer {access_token}"},
        data={
            "account_id": monzo["account_id"],
        },
    )
    logging.info(
        f""""Monzo balance:,
    {response.status_code},
    {response.reason},
    {response.content}"""
    )
    return response.json()


def post_monzo_balance_to_discord(
    url: str,
    monzo: dict,
    statein: func.InputStream,
    stateout: func.Out[func.InputStream],
):
    access_token = refresh_monzo_tokens(monzo=monzo, statein=statein, stateout=stateout)
    balance_json = monzo_get_balance(monzo=monzo, access_token=access_token)

    message = f"""⚖ Monzo Balance:
    💷 {balance_json["total_balance"]/100}{balance_json["currency"]}"""
    post_message_to_discord(url=url, message=message)


def main(
    request: func.HttpRequest,
    discordfile: func.InputStream,
    monzofile: func.InputStream,
    statein: func.InputStream,
    stateout: func.Out[func.InputStream],
) -> func.HttpResponse:
    logging.info("Received a request")

    logging.info(transaction_json := request.get_json())
    logging.info(monzo := json.load(monzofile))
    logging.info(discord := json.load(discordfile))

    post_transaction_to_discord(
        url=discordfile["url"], transaction_json=transaction_json
    )

    post_monzo_balance_to_discord(
        url=discordfile["url"], monzo=monzo, statein=statein, stateout=stateout
    )

    return func.HttpResponse(
        status_code=200,
    )
