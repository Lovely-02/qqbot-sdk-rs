import json
import threading
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

from qqbot_sdk_python import ApiError, Bot, BotError


class FakeQQ(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

    def reply(self, status, body):
        payload = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def do_GET(self):
        if self.path == "/users/@me" and self.headers.get("Authorization") == "QQBot fake":
            self.reply(200, {"id": "bot-1", "username": "测试机器人"})
        elif self.path == "/users/@me/guilds?limit=20":
            self.reply(200, [{"id": "guild-1", "name": "example"}])
        else:
            self.reply(404, {"code": 404, "message": "unexpected request"})

    def do_DELETE(self):
        if self.path == "/v2/users/user-1/messages/sent-1":
            self.reply(204, {})
        else:
            self.reply(404, {"code": 404, "message": "unexpected request"})

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        if self.path == "/app/getAppAccessToken":
            if body == {"appId": "test-id", "clientSecret": "test-secret"}:
                self.reply(200, {"access_token": "fake", "expires_in": 3600})
                return

        if self.path == "/v2/users/user-1/messages":
            if self.headers.get("Authorization") == "QQBot fake":
                if body == {"content": "你好", "msg_type": 0, "msg_id": "incoming", "msg_seq": 1}:
                    self.reply(200, {"id": "sent-1", "content": body["content"]})
                    return
                if body == {"content": "拒绝", "msg_type": 0}:
                    self.reply(403, {
                        "code": 40003,
                        "message": "forbidden",
                        "trace_id": "trace-1",
                        "data": {"reason": "test"},
                    })
                    return

        self.reply(404, {"code": 404, "message": "unexpected request"})


class SdkTest(unittest.IsolatedAsyncioTestCase):
    @classmethod
    def setUpClass(cls):
        cls.server = ThreadingHTTPServer(("127.0.0.1", 0), FakeQQ)
        cls.worker = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.worker.start()
        cls.addClassCleanup(cls.close_server)

    @classmethod
    def close_server(cls):
        cls.server.shutdown()
        cls.worker.join()
        cls.server.server_close()

    def bot(self):
        return Bot(
            "test-id", "test-secret",
            api_base_url=f"http://127.0.0.1:{self.server.server_port}",
        )

    async def test_get_bot_profile(self):
        profile = await self.bot().api.bot.me()
        self.assertEqual(profile["id"], "bot-1")
        self.assertEqual(profile["username"], "测试机器人")

    async def test_send_private_reply(self):
        user = self.bot().user("user-1")
        self.assertEqual(user.id, "user-1")
        response = await user.send("你好", msg_id="incoming")
        self.assertEqual(response["id"], "sent-1")
        self.assertEqual(response["content"], "你好")

    async def test_api_error_preserves_details(self):
        with self.assertRaises(ApiError) as raised:
            await self.bot().user("user-1").send("拒绝")
        error = raised.exception
        self.assertIsInstance(error, BotError)
        self.assertEqual((error.status, error.code, error.message), (403, 40003, "forbidden"))
        self.assertEqual(error.trace_id, "trace-1")
        self.assertEqual(error.data, {"reason": "test"})

    async def test_shared_dispatch_and_recall(self):
        bot = self.bot()
        api = bot.api
        self.assertEqual(set(vars(api)), {
            "bot", "channels", "groups", "guilds", "interactions", "materials",
            "menu", "messages", "panels", "users", "utility",
        })
        self.assertFalse(hasattr(bot, "call"))
        self.assertFalse(hasattr(bot, "me"))
        guilds = await api.bot.guilds(limit=20)
        self.assertEqual(guilds[0]["id"], "guild-1")
        response = await api.messages.reply_c2c("user-1", "你好", msg_id="incoming")
        self.assertEqual(response["id"], "sent-1")
        self.assertIsNone(await api.messages.delete_c2c("user-1", "sent-1"))

    async def test_grouped_media_dispatch(self):
        api = self.bot().api
        target = {"kind": "user", "id": "user-1"}
        with self.assertRaises(ValueError):
            await api.messages.upload_media(target, 1, b"", False)
        with self.assertRaises(ValueError):
            await api.messages.send_media(target, 1, b"", "你好")
        with self.assertRaises(ValueError):
            await api.messages.upload_media(target, 1, "invalid-url", False)

    async def test_gateway_lifecycle(self):
        bot = self.bot()
        bot.stop()
        self.assertIsNone(await bot.run(lambda event: self.fail(f"unexpected event {event}")))
        with self.assertRaises(BotError):
            await bot.run(lambda event: None)

        webhook = Bot("test-id", "test-secret", mode="webhook")
        with self.assertRaises(ValueError):
            await webhook.run(lambda event: None)
        with self.assertRaises(BotError):
            webhook.handle_webhook("123", "invalid", b"{}")

    def test_invalid_input(self):
        with self.assertRaises(ValueError):
            Bot("", "test-secret")
        with self.assertRaises(ValueError):
            self.bot().user(" ")
