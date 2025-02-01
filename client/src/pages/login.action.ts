import type { ActionFunction } from "react-router"
import { redirect } from "react-router-dom"

const action: ActionFunction = async ({ request }) => {
    const formData = await request.formData()
    const res = await fetch("/api/users/login", {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            username: formData.get("username"),
            password: formData.get("password"),
        }),
    })

    if (res.status === 200) return redirect("/")
    return { success: false, message: await res.text() }
}

export default action
