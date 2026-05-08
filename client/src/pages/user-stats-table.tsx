import Table from "react-bootstrap/Table"
import { useTranslation } from "react-i18next"
import { UserStats } from "../entities"

function percent(numerator: number, denominator: number): string {
    if (denominator <= 0) return "-"
    return `${Math.round((numerator / denominator) * 100)}%`
}

export default function UserStatsTable({ stats }: { stats: UserStats }) {
    const { t } = useTranslation()
    const hitSuccesses =
        stats.hits_guessed_correctly + stats.hits_stolen_successfully
    const hitAttempts =
        hitSuccesses +
        stats.hits_guessed_wrong +
        stats.hits_steal_attempts_failed
    const tokenAttempts = stats.tokens_earned + stats.tokens_missed

    return (
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
                    <td>{percent(stats.games_won, stats.games_played)}</td>
                </tr>
                <tr>
                    <td>{t("userStatsHitsGuessedCorrectly")}</td>
                    <td>{stats.hits_guessed_correctly}</td>
                </tr>
                <tr>
                    <td>{t("userStatsHitsGuessedWrong")}</td>
                    <td>{stats.hits_guessed_wrong}</td>
                </tr>
                <tr>
                    <td>{t("userStatsHitsStolenSuccessfully")}</td>
                    <td>{stats.hits_stolen_successfully}</td>
                </tr>
                <tr>
                    <td>{t("userStatsHitsStealAttemptsFailed")}</td>
                    <td>{stats.hits_steal_attempts_failed}</td>
                </tr>
                <tr>
                    <td>{t("userStatsHitSuccessRate")}</td>
                    <td>{percent(hitSuccesses, hitAttempts)}</td>
                </tr>
                <tr>
                    <td>{t("userStatsTokensEarned")}</td>
                    <td>{stats.tokens_earned}</td>
                </tr>
                <tr>
                    <td>{t("userStatsTokensMissed")}</td>
                    <td>{stats.tokens_missed}</td>
                </tr>
                <tr>
                    <td>{t("userStatsTokenSuccessRate")}</td>
                    <td>{percent(stats.tokens_earned, tokenAttempts)}</td>
                </tr>
            </tbody>
        </Table>
    )
}
