import { Application } from "https://deno.land/x/oak/mod.ts";
import { Router } from "https://deno.land/x/oak/mod.ts"

const app = new Application();

const router = new Router()

router.get("/", ({ response }: { response: any }) => {
    response.body = "hello noobs"
})


app.use(router.routes())
app.use(router.allowedMethods())

console.log("App running on localhost:3000");
await app.listen({ port: 3000 })
