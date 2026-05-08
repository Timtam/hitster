import { Helmet } from "@dr.pogodin/react-helmet"
import { useTranslation } from "react-i18next"
import { useLoaderData } from "react-router"
import { PublicUser, UserStats } from "../entities"
import FA from "../focus-anchor"
import UserStatsTable from "./user-stats-table"

export default function User() {
    const { t } = useTranslation()
    const [user, stats] = useLoaderData() as [PublicUser, UserStats]

    return (
        <>
            <Helmet>
                <title>{`${user.name} - Hitster`}</title>
            </Helmet>
            <FA>
                <h2>{user.name}</h2>
            </FA>
            <h3>{t("userStatsHeading")}</h3>
            <UserStatsTable stats={stats} />
        </>
    )
}
