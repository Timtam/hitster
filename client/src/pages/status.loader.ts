import { redirect } from "react-router"
import { HitsStatus } from "../entities"
import HitService from "../services/hits.service"

const loader = async (): Promise<ReturnType<typeof redirect> | HitsStatus> => {
    const hs = new HitService()
    let status = await hs.getStatus()

    if (status.finished === true) return redirect("/")
    else return status
}

export default loader
