import type { LoaderFunction } from "react-router"
import { Pack } from "../entities"
import HitService from "../services/hits.service"

const loader: LoaderFunction = async (): Promise<Pack[]> => {
    const hs = new HitService()

    const packs = await hs.getAllPacks()

    return packs
}

export default loader
