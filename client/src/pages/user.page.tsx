import { Helmet } from "@dr.pogodin/react-helmet"
import Table from "react-bootstrap/Table"
import { useTranslation } from "react-i18next"
import { useLoaderData } from "react-router"
import { PublicUser, UserStats } from "../entities"
import FA from "../focus-anchor"

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
            <Table responsive>
                <tbody>
                    <tr>
                        <td>{t("userStatsGamesPlayed")}</td>
                        <td>{stats.games_played}</td>
                    </tr>
                    <tr>
                        <td>{t("userStatsGamesWon")}</td>
                        <td>{stats.games_won}</td>
                    </tr>
                    <tr>
                        <td>{t("userStatsWinRate")}</td>
                        <td>
                            {stats.games_played > 0
                                ? `${Math.round(
                                      (stats.games_won / stats.games_played) *
                                          100,
                                  )}%`
                                : "-"}
                        </td>
                    </tr>
                    <tr>
                        <td>{t("userStatsHitsGuessedCorrectly")}</td>
                        <td>{stats.hits_guessed_correctly}</td>
                    </tr>
                    <tr>
                        <td>{t("userStatsTokensEarned")}</td>
                        <td>{stats.tokens_earned}</td>
                    </tr>
                </tbody>
            </Table>
        </>
    )
}
