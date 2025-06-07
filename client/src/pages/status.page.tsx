import { Helmet } from "@dr.pogodin/react-helmet"
import { useTranslation } from "react-i18next"
import { useLoaderData } from "react-router"
import { HitsStatus } from "../entities"
import { useRevalidateOnInterval } from "../hooks"

export default function Status() {
    const { t } = useTranslation()
    const status = useLoaderData() as HitsStatus

    useRevalidateOnInterval({ enabled: true, interval: 5000 })

    return (
        <>
            <Helmet>
                <title>{t("statusTitle") + " - Hitster"}</title>
            </Helmet>
            <h1>{t("statusHeading")}</h1>
            <p>{t("statusPreparingText")}</p>
            <label htmlFor="statusProgress">
                {t("statusProgressLabel", {
                    all: status.all,
                    downloaded: status.downloaded,
                })}
            </label>
            <progress
                value={Math.floor((status.downloaded * 100) / status.all)}
                max={100}
                id="statusProgress"
            />
        </>
    )
}
