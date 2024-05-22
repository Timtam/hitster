import sum from "ml-array-sum"
import { Helmet } from "react-helmet-async"
import { useTranslation } from "react-i18next"
import { useLoaderData } from "react-router-dom"
import HitService from "../services/hits.service"

export async function loader(): Promise<Record<string, number>> {
    let hs = new HitService()
    return await hs.getAllPacks()
}

export function Welcome() {
    let packs = useLoaderData() as Record<string, number>
    let { t } = useTranslation()

    return (
        <>
            <Helmet>
                <title>{t("welcome")} - Hitster</title>
            </Helmet>
            <h2>{t("welcome")}</h2>
            <p>{t("welcomeText")}</p>
            <h3>{t("features")}</h3>
            <ul>
                <li>{t("noRegistrationFeature")}</li>
                <li>{t("publicAndPrivateGamesFeature")}</li>
                <li>{t("localGamesFeature")}</li>
                <li>
                    {t("packsFeature", {
                        hits: sum(Object.values(packs)),
                        packs: Object.keys(packs).length,
                    })}
                </li>
                <li>{t("accessibilityFeature")}</li>
            </ul>
            <h3>{t("howToPlay")}</h3>
        </>
    )
}
