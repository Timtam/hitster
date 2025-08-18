//import { useLoaderData } from "react-router"
//import { Pack } from "../entities"
import { Helmet } from "@dr.pogodin/react-helmet"
import EventManager from "@lomray/event-manager"
import { useCallback, useEffect, useMemo, useState } from "react"
import Col from "react-bootstrap/Col"
import Form from "react-bootstrap/Form"
import Pagination from "react-bootstrap/Pagination"
import Row from "react-bootstrap/Row"
import Spinner from "react-bootstrap/Spinner"
import Table from "react-bootstrap/Table"
import { useTranslation } from "react-i18next"
import {
    HitSearchQuery,
    PaginatedHitsResponse,
    SortBy,
    SortDirection,
} from "../entities"
import { Events, NotificationData } from "../events"
import HitService from "../services/hits.service"

const PAGE_RANGE = 4
const PAGE_SIZE = 50
const PAGE_SKIPS = [10, 50, 100, 500, 1000]
const SEARCH_DELAY = 300

export default function Browser() {
    //const packs = useLoaderData() as Pack[]
    const hitService = useMemo(() => new HitService(), [])
    const { t } = useTranslation()
    const [searching, setSearching] = useState(true)
    const [hitResults, setHitResults] = useState<PaginatedHitsResponse>({
        results: [],
        start: 0,
        end: 0,
        total: 0,
    } satisfies PaginatedHitsResponse)
    const [query, setQuery] = useState("")
    const [searchTimer, setSearchTimer] = useState<ReturnType<
        typeof setTimeout
    > | null>(null)

    const search = useCallback(
        async (query: HitSearchQuery) => {
            setSearching(true)
            let results = await hitService.searchHits(query)
            EventManager.publish(Events.notification, {
                toast: false,
                interruptTts: true,
                text: t("showResults", {
                    start: results.start,
                    end: results.end,
                    total: results.total,
                }),
            } satisfies NotificationData)
            setHitResults(results)
            setSearching(false)
        },
        [hitService, setSearching, setHitResults],
    )

    const getPageCount = useCallback(
        () =>
            Math.floor(hitResults.total / PAGE_SIZE) +
            (hitResults.total % PAGE_SIZE > 0 ? 1 : 0),
        [hitResults.total],
    )

    const getCurrentPage = useCallback(
        () => Math.floor(hitResults.start / PAGE_SIZE) + 1,
        [hitResults.start],
    )

    useEffect(() => {
        ;(async () => {
            await search({
                start: 1,
                amount: PAGE_SIZE,
                sort_by: [
                    SortBy.Title,
                    SortBy.Artist,
                    SortBy.BelongsTo,
                    SortBy.Year,
                ],
                sort_direction: SortDirection.Ascending,
                query: "",
                packs: [],
            } satisfies HitSearchQuery)
        })()
    }, [])

    return (
        <>
            <Helmet>
                <title>{t("browseHits")} - Hitster</title>
            </Helmet>
            <h2>{t("browseHits")}</h2>
            <Row>
                <Col>
                    <search>
                        <h3>{t("search")}</h3>
                        <Form>
                            <Form.Group className="mb-2">
                                <Form.Label>{t("search")}</Form.Label>
                                <Form.Control
                                    type="search"
                                    placeholder={t("search")}
                                    value={query}
                                    onChange={(e) => {
                                        e.preventDefault()
                                        let q = e.currentTarget.value
                                        setQuery(q)
                                        if (searchTimer !== null)
                                            clearTimeout(searchTimer)
                                        setSearchTimer(
                                            setTimeout(async () => {
                                                await search({
                                                    query: q,
                                                    start: 1,
                                                    amount: PAGE_SIZE,
                                                    sort_by: [
                                                        SortBy.Title,
                                                        SortBy.Artist,
                                                        SortBy.BelongsTo,
                                                        SortBy.Year,
                                                    ],
                                                    sort_direction:
                                                        SortDirection.Ascending,
                                                    packs: [],
                                                } satisfies HitSearchQuery)
                                                setSearchTimer(null)
                                            }, SEARCH_DELAY),
                                        )
                                    }}
                                />
                            </Form.Group>
                        </Form>
                    </search>
                </Col>
            </Row>
            <Row>
                <Col>
                    {searching ? (
                        <Spinner animation="border" role="status">
                            <span className="visually-hidden">
                                {t("loading")}
                            </span>
                        </Spinner>
                    ) : (
                        ""
                    )}
                    <Table responsive>
                        <thead>
                            <tr>
                                <th>{t("title")}</th>
                                <th>{t("artist")}</th>
                                <th>{t("year")}</th>
                                <th>{t("belongsTo")}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {hitResults.results.map((hit) => (
                                <tr key={hit.id}>
                                    <td>{hit.title}</td>
                                    <td>{hit.artist}</td>
                                    <td>{hit.year}</td>
                                    <td>{hit.belongs_to}</td>
                                </tr>
                            ))}
                        </tbody>
                    </Table>
                    <Pagination>
                        <Pagination.Item
                            disabled={getCurrentPage() === 1}
                            onClick={async () =>
                                await search({
                                    start: 1,
                                    amount: PAGE_SIZE,
                                    sort_by: [
                                        SortBy.Title,
                                        SortBy.Artist,
                                        SortBy.BelongsTo,
                                        SortBy.Year,
                                    ],
                                    sort_direction: SortDirection.Ascending,
                                    query: query,
                                    packs: [],
                                } satisfies HitSearchQuery)
                            }
                        >
                            {t("first")}
                        </Pagination.Item>
                        <Pagination.Item
                            disabled={getCurrentPage() === 1}
                            onClick={async () =>
                                await search({
                                    start:
                                        PAGE_SIZE * (getCurrentPage() - 2) + 1,
                                    amount: PAGE_SIZE,
                                    sort_by: [
                                        SortBy.Title,
                                        SortBy.Artist,
                                        SortBy.BelongsTo,
                                        SortBy.Year,
                                    ],
                                    sort_direction: SortDirection.Ascending,
                                    query: query,
                                    packs: [],
                                } satisfies HitSearchQuery)
                            }
                        >
                            {t("previous")}
                        </Pagination.Item>
                        {Array.from(
                            { length: getPageCount() },
                            (_, i) => i,
                        ).flatMap((i) => {
                            let pages = []

                            if (
                                (i < getCurrentPage() + PAGE_RANGE &&
                                    i >= getCurrentPage() - PAGE_RANGE - 1) ||
                                PAGE_SKIPS.includes(
                                    Math.abs(i - getCurrentPage() + 1),
                                )
                            )
                                pages.push(
                                    <Pagination.Item
                                        key={`page-${i + 1}`}
                                        active={i + 1 === getCurrentPage()}
                                        activeLabel={t("current")}
                                        onClick={async () =>
                                            await search({
                                                start: PAGE_SIZE * i + 1,
                                                amount: PAGE_SIZE,
                                                sort_by: [
                                                    SortBy.Title,
                                                    SortBy.Artist,
                                                    SortBy.BelongsTo,
                                                    SortBy.Year,
                                                ],
                                                sort_direction:
                                                    SortDirection.Ascending,
                                                query: query,
                                                packs: [],
                                            } satisfies HitSearchQuery)
                                        }
                                    >
                                        {i + 1}
                                    </Pagination.Item>,
                                )

                            if (
                                i === getCurrentPage() + PAGE_RANGE - 1 ||
                                PAGE_SKIPS.includes(
                                    Math.abs(i - getCurrentPage() + 1),
                                )
                            )
                                pages.push(
                                    <Pagination.Item as="div">
                                        ...
                                    </Pagination.Item>,
                                )

                            return pages
                        })}
                        <Pagination.Item
                            disabled={getCurrentPage() === getPageCount()}
                            onClick={async () =>
                                await search({
                                    start: PAGE_SIZE * getCurrentPage() + 1,
                                    amount: PAGE_SIZE,
                                    sort_by: [
                                        SortBy.Title,
                                        SortBy.Artist,
                                        SortBy.BelongsTo,
                                        SortBy.Year,
                                    ],
                                    sort_direction: SortDirection.Ascending,
                                    query: query,
                                    packs: [],
                                } satisfies HitSearchQuery)
                            }
                        >
                            {t("next")}
                        </Pagination.Item>
                        <Pagination.Item
                            disabled={getCurrentPage() === getPageCount()}
                            onClick={async () =>
                                await search({
                                    start: PAGE_SIZE * (getPageCount() - 1) + 1,
                                    amount: PAGE_SIZE,
                                    sort_by: [
                                        SortBy.Title,
                                        SortBy.Artist,
                                        SortBy.BelongsTo,
                                        SortBy.Year,
                                    ],
                                    sort_direction: SortDirection.Ascending,
                                    query: query,
                                    packs: [],
                                } satisfies HitSearchQuery)
                            }
                        >
                            {t("last")}
                        </Pagination.Item>
                    </Pagination>
                </Col>
            </Row>
        </>
    )
}
