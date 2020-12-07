# %%
import json
import logging
from dataclasses import dataclass
from datetime import datetime

import azure.functions as func
import dateutil.parser as dateparse
import requests


class Base:
    def __iter__(self):
        yield from self.__dict__.items()

    @classmethod
    def from_json(cls, s: str):
        d = json.loads(s)

        return cls.from_dict(d)

    @classmethod
    def from_read(cls, file):
        d = json.load(file)

        return cls.from_dict(d)

    @classmethod
    def from_dict(cls, d: dict):
        # Intersection
        keys = d.keys() & cls.__dataclass_fields__.keys()

        # Try and build from that
        try:
            return cls(**{k: d[k] for k in keys})
        except Exception as e:
            logging.info(
                f"File provided {d.keys()}, expected {cls.__dataclass_fields__.keys()}, intersected at {keys}"
            )
            raise e

    def to_json(self):
        return json.dumps(dict(self))


@dataclass
class MonzoConfig(Base):
    account_id: str
    client_id: str
    client_secret: str


@dataclass
class MonzoState(Base):
    access_token: str
    refresh_token: str


@dataclass
class DiscordConfig(Base):
    url: str

    def post_message(self, message: str):
        response = requests.post(self.url, json={"content": message})
        if not response.ok:
            logging.info(f"Us: {response.request.body}")
            logging.info(
                f"Discord: {response.status_code},{response.reason},\n{response.content}"
            )
            raise Exception("Discord not happy")


# %%
@dataclass
class Transaction:
    amount: float
    created: datetime
    merchant_name: str

    @classmethod
    def from_request(cls, request: func.HttpRequest):
        try:
            d = request.get_json()

            return cls(
                amount=d["data"]["amount"],
                created=dateparse.parse(d["data"]["created"]),
                merchant_name=d["data"]["merchant"]["name"],
            )

        except Exception as e:
            logging.info(
                f"Transaction: {request.method} {request.url}\n{request.get_body()}"
            )
            raise e

    def post_message(self, discord: DiscordConfig):
        discord.post_message(
            f"""💸 New transaction :
    📅 {str(self.created)}
    💷 {self.amount/100} ({"📉" if self.amount < 0 else "📈"})
    📌 {self.merchant_name}"""
        )


@dataclass
class MonzoBalance(Base):
    balance: int
    total_balance: int
    currency: int
    spend_today: int

    @classmethod
    def from_web(cls, account_id, access_token):
        response = requests.get(
            url="https://api.monzo.com/balance",
            headers={"Authorization": f"Bearer {access_token}"},
            params={
                "account_id": account_id,
            },
        )
        try:
            return cls.from_dict(response.json())
        except Exception as e:
            logging.info(f"Us: {response.request.headers}{response.request.body}")
            logging.info(
                f""""Monzo balance:,
            {response.status_code},
            {response.reason},
            {response.content}"""
            )
            raise e

    def post_message(self, discord: DiscordConfig):
        discord.post_message(f"""⚖ Account balance:\n    {100}""")
        logging.info(self.total_balance)
