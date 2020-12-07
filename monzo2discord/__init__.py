import json
import logging
import structures

import azure.functions as func
import requests


# fragile
def get_access_token(
    monzo: structures.MonzoConfig,
    statein: func.InputStream,
    stateout: func.Out[func.InputStream],
):
    old_state = structures.MonzoState.from_read(statein)
    logging.info(f"old state: {old_state}")

    response = requests.post(
        url="https://api.monzo.com/oauth2/token",
        data={
            "grant_type": "refresh_token",
            "client_id": monzo.client_id,
            "client_secret": monzo.client_secret,
            "refresh_token": old_state.refresh_token,
        },
    )

    try:
        parsed = response.json()
        new_state = structures.MonzoState(
            access_token=parsed["access_token"], refresh_token=parsed["refresh_token"]
        )
        stateout.set(new_state.to_json())
    except Exception as e:
        logging.error(f"Us: {response.request.headers}{response.request.body}")
        logging.error(
            f""""Monzo refresh:,
        {response.status_code},
        {response.reason},
        {response.content}"""
        )
        raise e

    logging.info(f"new state: {new_state}")
    return new_state.access_token


def main(
    request: func.HttpRequest,
    discordfile: func.InputStream,
    monzofile: func.InputStream,
    statein: func.InputStream,
    stateout: func.Out[func.InputStream],
) -> func.HttpResponse:
    logging.info("Received a request")

    monzo = structures.MonzoConfig.from_read(monzofile)
    discord = structures.DiscordConfig.from_read(discordfile)

    if request.method == "POST":
        transaction = structures.Transaction.from_request(request)
        transaction.post_message(discord)

    access_token = get_access_token(monzo, statein, stateout)
    balance = structures.MonzoBalance.from_web(monzo.account_id, access_token)
    balance.post_message(discord)

    return func.HttpResponse(
        status_code=200,
    )
