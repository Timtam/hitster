import { redirect } from "react-router"
import HitService from "./services/hits.service"

const loader = async (): Promise<ReturnType<typeof redirect> | undefined> => {
    const hs = new HitService()
    let status = await hs.getStatus()
    if (!status.finished) return redirect("/status")
}

export default loader
