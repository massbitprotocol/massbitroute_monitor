#!/usr/bin/python3
# -*- coding: utf-8 -*-

import os
import time

from redis import StrictRedis

from flask import Flask, abort, jsonify, request

LIFETIME = 90


class ReverseProxied(object):
    def __init__(self, app):
        self.app = app

    def __call__(self, environ, start_response):
        script_name = environ.get("HTTP_X_SCRIPT_NAME", "")
        if script_name:
            environ["SCRIPT_NAME"] = script_name
            path_info = environ["PATH_INFO"]
            if path_info.startswith(script_name):
                environ["PATH_INFO"] = path_info[len(script_name) :]

        scheme = environ.get("HTTP_X_SCHEME", "")
        if scheme:
            environ["wsgi.url_scheme"] = scheme
        return self.app(environ, start_response)


app = Flask("check_mk_push_agent_server")
proxy_app = ReverseProxied(app)
# redis = StrictRedis(unix_socket_path="/app/sites/cdn/tmp/redis_mk.sock")
redis = StrictRedis(unix_socket_path="/omd/sites/mbr/tmp/run/redis")

tokens = {}


def load_tokens():
    with open(os.environ["TOKEN_FILE"]) as token_file:
        for line in token_file:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            token, hostname = line.split()
            yield token, hostname


@app.route("/push/<token_arg>", methods=["POST"])
def push(token_arg):
    """Push check_mk_agent data."""

    token_r = token_arg.split(":")
    token = token_r[0]

    if token not in tokens:
        abort(404)
    hostname = tokens[token]

    if len(token_r) == 2:
        token_i = token_r[1]
        hostname = ":".join([hostname, token_i])

    redis.setex(
        ":".join(["check_mk_push_agent", "data", hostname]), LIFETIME, request.data
    )
    redis.hset("check_mk_push_agent:last_seen", hostname, time.time())
    return jsonify(dict(status="ok"))


if __name__ == "__main__":
    # from waitress import serve

    tokens = dict(load_tokens())
    print(tokens)
    print(redis)
    # serve(app, debug=False, threaded=True, host="127.0.0.1", port=18889)
    app.run(debug=False, threaded=True, host="127.0.0.1", port=18889)
