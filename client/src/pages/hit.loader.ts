import type { LoaderFunction } from "react-router"
import { FullHit, HitQueryPart, HitStats, Pack } from "../entities"
import HitService from "../services/hits.service"

const loader: LoaderFunction = async ({
    params,
}): Promise<[FullHit, Pack[], HitStats]> => {
    const hs = new HitService()

    if (params.hitId !== undefined) {
        const hit = await hs.get(params.hitId, [HitQueryPart.Issues])

        if (hit === undefined)
            throw { message: "hit id not found", status: 404 }

        const [packs, stats] = await Promise.all([
            hs.getAllPacks(),
            hs.getStats(params.hitId),
        ])

        return [hit, packs, stats]
    }
    throw { message: "internal api error", status: 500 }
}

export default loader
