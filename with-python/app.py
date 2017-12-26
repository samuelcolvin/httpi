from time import time
from random import random

import asyncpg
import ujson
from aiohttp import web

STEPS = 100


async def sql(request):
    start = time()
    v = 0
    async with request.app['pool'].acquire() as conn:
        for _ in range(STEPS):
            v += await conn.fetchval('SELECT (random() ^ 2 + random() ^ 2) < 1')
    text = ujson.dumps({
        'pi': v / STEPS * 4,
        'sql_exec_time': time() - start,
    })
    return web.Response(text=text, content_type='application/json')


async def fast(request):
    # start = time()
    v = 0
    for _ in range(STEPS):
        a, b = random(), random()
        v += (a ** 2 + b ** 2) < 1
    text = ujson.dumps({
        'pi': v / STEPS * 4,
        'sql_exec_time': 0,
        # 'sql_exec_time': time() - start,
    })
    return web.Response(text=text, content_type='application/json')


async def startup(app):
    app['pool'] = await asyncpg.create_pool(dsn='postgres://postgres:waffle@localhost:5432')


app = web.Application()
app.on_startup.append(startup)
app.router.add_get('/', sql)
app.router.add_get('/fast/', fast)

if __name__ == '__main__':
    web.run_app(app, port=8000, access_log=None, access_log_format=None)
