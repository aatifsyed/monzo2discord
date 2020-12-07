# %%
# Imports
import enum
from collections import namedtuple
from urllib import parse

import pyperclip
import requests

import structures

# %%
# Session setup
class DestinationUrl(enum.Enum):
    LIVE = "https://monzo2discord.azurewebsites.net/api/monzo2discord"
    TEST = "http://localhost:7071/api/monzo2discord"


with open("data/monzo.json") as f:
    config = structures.MonzoConfig.from_read(f)
with open("data/state.json") as f:
    state = structures.MonzoState.from_read(f)

session = requests.Session()
session.headers.update({"Authorization": f"Bearer {state.access_token}"})

# %%
# Webhooks
def get_webhooks(session: requests.Session, config: structures.MonzoConfig):
    response = session.get(
        url="https://api.monzo.com/webhooks",
        params={"account_id": config.account_id},
    )

    if not response.ok:
        print(response.content)
        response.raise_for_status()

    return response.json()["webhooks"]


def del_webhook(session: requests.Session, webhook_id: str):
    response = session.delete(
        url=f"https://api.monzo.com/webhooks/{webhook_id}",
    )

    if not response.ok:
        print(response.content)
        response.raise_for_status()


def set_webhook(session: requests.Session, config: structures.MonzoConfig, url: str):
    response = session.post(
        url="https://api.monzo.com/webhooks",
        data={"account_id": config.account_id, "url": url},
    )

    if not response.ok:
        print(response.content)
        response.raise_for_status()


# %%
# Test endpoint
def test_transaction(url: str):
    example_body = {
        "type": "transaction.created",
        "data": {
            "account_id": "acc_00008gju41AHyfLUzBUk8A",
            "amount": -350,
            "created": "2015-09-04T14:28:40Z",
            "currency": "GBP",
            "description": "Ozone Coffee Roasters",
            "id": "tx_00008zjky19HyFLAzlUk7t",
            "category": "eating_out",
            "is_load": False,
            "settled": "2015-09-05T14:28:40Z",
            "merchant": {
                "address": {
                    "address": "98 Southgate Road",
                    "city": "London",
                    "country": "GB",
                    "latitude": 51.54151,
                    "longitude": -0.08482400000002599,
                    "postcode": "N1 3JD",
                    "region": "Greater London",
                },
                "created": "2015-08-22T12:20:18Z",
                "group_id": "grp_00008zIcpbBOaAr7TTP3sv",
                "id": "merch_00008zIcpbAKe8shBxXUtl",
                "logo": "https://pbs.twimg.com/profile_images/527043602623389696/68_SgUWJ.jpeg",
                "emoji": "🍞",
                "name": "The De Beauvoir Deli Co.",
                "category": "eating_out",
            },
        },
    }
    response = requests.post(url=url, json=example_body)

    if not response.ok:
        print(response.content)
        response.raise_for_status()


def test_balance(url: str):
    response = requests.get(url=url)

    if not response.ok:
        print(response.content)
        response.raise_for_status()


# %%
# Authorization
def start_auth(config: structures.MonzoConfig):
    query = parse.urlencode(
        {
            "client_id": config.client_id,
            "redirect_uri": "https://aatifsyed.uk",
            "response_type": "code",
            "state": 1234,
        }
    )

    Url = namedtuple("Url", ["scheme", "netloc", "path", "params", "query", "fragment"])

    pre_auth_url = Url(
        scheme="https",
        netloc="auth.monzo.com",
        path="",
        params="",
        query=query,
        fragment="",
    )

    url = parse.urlunparse(pre_auth_url)
    pyperclip.copy(url)
    return url


def new_monzostate(authorized_url: str, config: structures.MonzoConfig):
    authorization_code = parse.parse_qs(parse.urlparse(authorized_url).query)["code"]
    response = requests.post(
        url="https://api.monzo.com/oauth2/token",
        data={
            "grant_type": "authorization_code",
            "client_id": config.client_id,
            "client_secret": config.client_secret,
            "redirect_uri": "https://aatifsyed.uk",
            "code": authorization_code,
        },
    )

    if not response.ok:
        print(response.content)
        response.raise_for_status()

    return structures.MonzoState.from_dict(response.json())


def write_monzostate(state: structures.MonzoState):
    with open("data/state.json", "w") as f:
        f.write(state.to_json())


# %%
def ping(session: requests.Session):
    response = session.get("https://api.monzo.com/ping/whoami")
    print(response.json())


# %%
