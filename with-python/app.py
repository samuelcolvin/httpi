import os
from random import SystemRandom

import asyncpg
import ujson
from aiohttp.web import Application, Response, run_app

DEFAULT_STEPS = 100
random = SystemRandom().random


def get_steps(request):
    try:
        return int(request.query['steps'])
    except (ValueError, KeyError):
        return DEFAULT_STEPS


async def in_sql(request):
    steps = get_steps(request)
    v = 0
    async with request.app['pool'].acquire() as conn:
        stmt = await conn.prepare('SELECT (random() ^ 2 + random() ^ 2) < 1')
        for _ in range(steps):
            v += await stmt.fetchval()
    return Response(
        text=ujson.dumps({'pi': v / steps * 4}),
        content_type='application/json'
    )


async def native(request):
    steps = get_steps(request)
    v = 0
    for _ in range(steps):
        a, b = random(), random()
        v += (a * a + b * b) < 1
    return Response(
        text=ujson.dumps({'pi': v / steps * 4}),
        content_type='application/json'
    )


async def startup(app):
    dsn = os.getenv('DB_DSN')
    app['pool'] = await asyncpg.create_pool(dsn=dsn)


app = Application()
app.on_startup.append(startup)
app.router.add_get('/sql/', sql)
app.router.add_get('/native/', native)

if __name__ == '__main__':
    run_app(app, port=8000, access_log=None, access_log_format=None)
