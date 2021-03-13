import requests_oauthlib
from requests_oauthlib import OAuth2Session
import requests
import pytest
import logging

logger = logging.getLogger(__name__)


@pytest.fixture
def client_id():
    return r""


@pytest.fixture
def client_secret():
    return r""


def test_oauth(client_id: str, client_secret: str):
    oauth = OAuth2Session(client_id=client_id, redirect_uri="https://example.com")
    logger.info(oauth.authorization_url(r"https://auth.monzo.com/"))
    url = input("url")
    token = oauth.fetch_token(
        token_url="https://api.monzo.com/oauth2/token",
        authorization_response=url,
        client_secret=client_secret,
    )
    logger.info(token)
    logger.info(oauth.get("https://api.monzo.com/ping/whoami"))
