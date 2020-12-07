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
        new_state.to_write(stateout)
        return new_state.access_token
    except:
        logging.info(
            f""""Monzo refresh:,
        {response.status_code},
        {response.reason},
        {response.content}"""
        )


def main(
    request: func.HttpRequest,
    discordfile: func.InputStream,
    monzofile: func.InputStream,
    statein: func.InputStream,
    stateout: func.Out[func.InputStream],
) -> func.HttpResponse:
    logging.info("Received a request")

    transaction = structures.Transaction.from_request(request)
    monzo = structures.MonzoConfig.from_read(monzofile)
    discord = structures.DiscordConfig.from_read(discordfile)

    transaction.post_message(discord)

    access_token = get_access_token(monzo, statein, stateout)
    balance = structures.MonzoBalance.from_web(monzo.account_id, access_token)
    balance.post_message(discord)

    return func.HttpResponse(
        status_code=200,
    )
