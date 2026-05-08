import { Helmet } from "@dr.pogodin/react-helmet"
import { useTranslation } from "react-i18next"
import { useLoaderData } from "react-router"
import { PublicUser, UserStats } from "../entities"
import FA from "../focus-anchor"
import UserStatsTable from "./user-stats-table"

export default function Player() {
    const { t } = useTranslation()
    const [player, stats] = useLoaderData() as [PublicUser, UserStats]

    return (
        <>
            <Helmet>
                <title>{`${player.name} - Hitster`}</title>
            </Helmet>
            <FA>
                <h2>{player.name}</h2>
            </FA>
            <h3>{t("userStatsHeading")}</h3>
            <UserStatsTable stats={stats} />
        </>
    )
}
