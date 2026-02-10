import {
    closestCenter,
    DndContext,
    DragEndEvent,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
} from "@dnd-kit/core"
import {
    arrayMove,
    SortableContext,
    sortableKeyboardCoordinates,
    useSortable,
    verticalListSortingStrategy,
} from "@dnd-kit/sortable"
import { CSS } from "@dnd-kit/utilities"
import { Helmet } from "@dr.pogodin/react-helmet"
import EventManager from "@lomray/event-manager"
import { toCamelCase, toPascalCase } from "js-convert-case"
import { ReactNode, useCallback, useEffect, useMemo, useState } from "react"
import Button from "react-bootstrap/Button"
import Col from "react-bootstrap/Col"
import Form from "react-bootstrap/Form"
import Pagination from "react-bootstrap/Pagination"
import Row from "react-bootstrap/Row"
import Spinner from "react-bootstrap/Spinner"
import Table from "react-bootstrap/Table"
import { useTranslation } from "react-i18next"
import { Link, useLoaderData, useSearchParams } from "react-router"
import { useImmer } from "use-immer"
import { useContext } from "../context"
import {
    HitQueryPart,
    HitSearchQuery,
    Pack,
    PaginatedHitsResponse,
    SortBy,
    SortDirection,
} from "../entities"
import {
    Events,
    IssueCreatedData,
    IssueDeletedData,
    NotificationData,
} from "../events"
import FA from "../focus-anchor"
import { useRevalidate } from "../hooks"
import HitService from "../services/hits.service"
import PacksModal from "./browser/packs"

const PAGE_RANGE = 4
const PAGE_SIZE = 50
const PAGE_SKIPS = [10, 50, 100, 500, 1000]
const SEARCH_DELAY = 300
const SORT_BY_INDEX: SortBy[] = [
    SortBy.Title,
    SortBy.Artist,
    SortBy.BelongsTo,
    SortBy.Year,
]

export function SortableItem(props: { id: number; children: ReactNode }) {
    const { t } = useTranslation()
    const { attributes, listeners, setNodeRef, transform, transition } =
        useSortable({ id: props.id })

    const style = {
        transform: CSS.Transform.toString(transform),
        transition,
    }

    attributes["aria-roledescription"] = t("sortable")

    return (
        <div ref={setNodeRef} style={style} {...attributes} {...listeners}>
            {props.children}
        </div>
    )
}

export default function Browser() {
    const availablePacks = useLoaderData() as Pack[]
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
    const [sortByItems, setSortByItems] = useState([0, 1, 2, 3])
    const sensors = useSensors(
        useSensor(PointerSensor),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates,
        }),
    )
    const [sortDirection, setSortDirection] = useState(SortDirection.Ascending)
    const [showPacksModal, setShowPacksModal] = useImmer<boolean[]>([])
    const [packs, setPacks] = useState<string[]>([])
    const [showPackFilter, setShowPackFilter] = useState(false)
    const revalidate = useRevalidate()
    const { user } = useContext()
    const canReadIssues = useMemo(
        () => user?.permissions.read_issues === true,
        [user],
    )
    const issueParts = useMemo(
        () => (canReadIssues ? [HitQueryPart.Issues] : undefined),
        [canReadIssues],
    )
    const [searchParams, setSearchParams] = useSearchParams()
    const [page, setPage] = useState(1)

    const search = useCallback(
        async (query: HitSearchQuery) => {
            setSearching(true)
            const results = await hitService.searchHits({
                ...query,
                parts: issueParts,
            })
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
            setShowPacksModal(
                Array.from({ length: results.results.length }, () => false),
            )
        },
        [
            hitService,
            setSearching,
            setHitResults,
            setShowPacksModal,
            t,
            issueParts,
        ],
    )

    const getPageCount = useCallback(
        () =>
            Math.floor(hitResults.total / PAGE_SIZE) +
            (hitResults.total % PAGE_SIZE > 0 ? 1 : 0),
        [hitResults.total],
    )

    const mapSortByIndexToElement = useCallback(
        (i: number) => (
            <SortableItem id={i} key={`sort-by-${i}`}>
                <li key={"sort-by-" + SORT_BY_INDEX[i]}>
                    {t(toCamelCase(SORT_BY_INDEX[i]))}
                </li>
            </SortableItem>
        ),
        [t],
    )

    const handleDragEnd = useCallback(
        (event: DragEndEvent) => {
            const { active, over } = event

            if (over === null) return

            if (active.id !== over.id) {
                const oldIndex = sortByItems.indexOf(active.id as number)
                const newIndex = sortByItems.indexOf(over.id as number)

                const sortBy = arrayMove(sortByItems, oldIndex, newIndex)

                setSearchParams((p) => {
                    p.delete("sort_by")
                    sortBy
                        .map((i) => SORT_BY_INDEX[i])
                        .forEach((s) => p.append("sort_by", s))
                    p.set("p", "1")
                    return p
                })
                ;(async () =>
                    await search({
                        query: query,
                        start: 1,
                        amount: PAGE_SIZE,
                        sort_by: sortBy.map((i) => SORT_BY_INDEX[i]),
                        sort_direction: sortDirection,
                        packs: packs,
                    } satisfies HitSearchQuery))()
                setSortByItems(sortBy)
            }
        },
        [
            sortByItems,
            setSortByItems,
            search,
            query,
            sortDirection,
            packs,
            setSearchParams,
        ],
    )

    useEffect(() => {
        const p: string = searchParams.get("p") ?? `${page}`
        const q: string = searchParams.get("q") ?? query
        const packs = searchParams.getAll("pack")
        const sortDirection =
            SortDirection[
                toPascalCase(
                    searchParams.get("sort_direction") ?? "ascending",
                ) as keyof typeof SortDirection
            ] ?? SortDirection.Ascending

        let sortCriteria = searchParams.getAll("sort_by")

        if (sortCriteria.length === 0) sortCriteria = SORT_BY_INDEX
        else
            sortCriteria = sortCriteria.reduce((acc, i) => {
                const crit = SortBy[toPascalCase(i) as keyof typeof SortBy]
                if (crit) acc.push(crit)
                return acc
            }, [] as SortBy[])

        setQuery(q)
        setPacks(
            availablePacks.filter((p) => packs.includes(p.id)).map((p) => p.id),
        )
        setSortDirection(sortDirection)
        setSortByItems(
            (sortCriteria as SortBy[]).map(
                (crit) => SORT_BY_INDEX.indexOf(crit)!,
            ),
        )
        setPage(parseInt(p, 10))
        ;(async () => {
            await search({
                start: (parseInt(p, 10) - 1) * PAGE_SIZE + 1,
                amount: PAGE_SIZE,
                sort_by: sortCriteria as SortBy[],
                sort_direction: sortDirection,
                query: q,
                packs: packs,
            } satisfies HitSearchQuery)
        })()
    }, [setQuery, setPacks, setSortDirection, setSortByItems, setPage, search])

    useEffect(() => {
        if (!canReadIssues) return

        const unsubscribeIssueCreated = EventManager.subscribe(
            Events.issueCreated,
            (e: IssueCreatedData) => {
                setHitResults((current) => {
                    const resultIndex = current.results.findIndex(
                        (hit) => hit.id === e.issue.hit_id,
                    )

                    if (resultIndex === -1) return current

                    const hit = current.results[resultIndex]
                    const existingIssues = hit.issues ?? []

                    if (
                        existingIssues.some((issue) => issue.id === e.issue.id)
                    ) {
                        return current
                    }

                    const results = [...current.results]
                    results[resultIndex] = {
                        ...hit,
                        issues: [...existingIssues, e.issue],
                    }

                    return {
                        ...current,
                        results,
                    }
                })
            },
        )

        const unsubscribeIssueDeleted = EventManager.subscribe(
            Events.issueDeleted,
            (e: IssueDeletedData) => {
                setHitResults((current) => {
                    const resultIndex = current.results.findIndex(
                        (hit) => hit.id === e.hitId,
                    )

                    if (resultIndex === -1) return current

                    const hit = current.results[resultIndex]
                    const existingIssues = hit.issues ?? []
                    const issues = existingIssues.filter(
                        (issue) => issue.id !== e.issueId,
                    )

                    if (issues.length === existingIssues.length) {
                        return current
                    }

                    const results = [...current.results]
                    results[resultIndex] = {
                        ...hit,
                        issues,
                    }

                    return {
                        ...current,
                        results,
                    }
                })
            },
        )

        return () => {
            unsubscribeIssueCreated()
            unsubscribeIssueDeleted()
        }
    }, [canReadIssues])

    return (
        <>
            <Helmet>
                <title>{t("browseHits")} - Hitster</title>
            </Helmet>
            <FA>
                <h2>{t("browseHits")}</h2>
            </FA>
            <Row>
                <Col>
                    <search>
                        <h3>{t("search")}</h3>
                        <Form onSubmit={(e) => e.preventDefault()}>
                            <Form.Group className="mb-2">
                                <Form.Control
                                    type="search"
                                    placeholder={t("search")}
                                    value={query}
                                    onChange={(e) => {
                                        e.preventDefault()
                                        const q = e.currentTarget.value
                                        setQuery(q)
                                        if (searchTimer !== null)
                                            clearTimeout(searchTimer)
                                        setSearchTimer(
                                            setTimeout(async () => {
                                                setSearchParams((p) => {
                                                    p.set("q", q)
                                                    p.set("p", "1")
                                                    return p
                                                })
                                                await search({
                                                    query: q,
                                                    start: 1,
                                                    amount: PAGE_SIZE,
                                                    sort_by: sortByItems.map(
                                                        (i) => SORT_BY_INDEX[i],
                                                    ),
                                                    sort_direction:
                                                        sortDirection,
                                                    packs: packs,
                                                } satisfies HitSearchQuery)
                                                setSearchTimer(null)
                                            }, SEARCH_DELAY),
                                        )
                                    }}
                                />
                            </Form.Group>
                            <Form.Group className="mb-2">
                                <fieldset>
                                    <legend>{t("sortBy")}</legend>
                                    <span className="visually-hidden">
                                        {t("reorderHintARIA")}
                                    </span>
                                    <span aria-hidden={true}>
                                        {t("reorderHintVisual")}
                                    </span>
                                    <ul>
                                        <DndContext
                                            accessibility={{
                                                announcements: {
                                                    onDragStart: ({ active }) =>
                                                        t("dragStart", {
                                                            draggable: t(
                                                                toCamelCase(
                                                                    SORT_BY_INDEX[
                                                                        active.id as number
                                                                    ],
                                                                ),
                                                            ),
                                                        }),
                                                    onDragOver: ({
                                                        active,
                                                        over,
                                                    }) => {
                                                        if (over) {
                                                            return t(
                                                                "dragOverDroppable",
                                                                {
                                                                    draggable:
                                                                        t(
                                                                            toCamelCase(
                                                                                SORT_BY_INDEX[
                                                                                    active.id as number
                                                                                ],
                                                                            ),
                                                                        ),
                                                                    droppable:
                                                                        t(
                                                                            toCamelCase(
                                                                                SORT_BY_INDEX[
                                                                                    over.id as number
                                                                                ],
                                                                            ),
                                                                        ),
                                                                },
                                                            )
                                                        }
                                                        return t(
                                                            "dragNotOverDroppable",
                                                            {
                                                                draggable: t(
                                                                    toCamelCase(
                                                                        SORT_BY_INDEX[
                                                                            active.id as number
                                                                        ],
                                                                    ),
                                                                ),
                                                            },
                                                        )
                                                    },
                                                    onDragEnd: ({
                                                        active,
                                                        over,
                                                    }) => {
                                                        if (over) {
                                                            return t(
                                                                "dragEndOverDroppable",
                                                                {
                                                                    draggable:
                                                                        t(
                                                                            toCamelCase(
                                                                                SORT_BY_INDEX[
                                                                                    active.id as number
                                                                                ],
                                                                            ),
                                                                        ),
                                                                    droppable:
                                                                        t(
                                                                            toCamelCase(
                                                                                SORT_BY_INDEX[
                                                                                    over.id as number
                                                                                ],
                                                                            ),
                                                                        ),
                                                                },
                                                            )
                                                        }

                                                        return t(
                                                            "dragEndNotOverDroppable",
                                                            {
                                                                draggable: t(
                                                                    toCamelCase(
                                                                        SORT_BY_INDEX[
                                                                            active.id as number
                                                                        ],
                                                                    ),
                                                                ),
                                                            },
                                                        )
                                                    },
                                                    onDragCancel: ({
                                                        active,
                                                    }) =>
                                                        t("dragCanceled", {
                                                            draggable: t(
                                                                toCamelCase(
                                                                    SORT_BY_INDEX[
                                                                        active.id as number
                                                                    ],
                                                                ),
                                                            ),
                                                        }),
                                                },
                                                screenReaderInstructions: {
                                                    draggable: t(
                                                        "reorderInstructions",
                                                    ),
                                                },
                                            }}
                                            sensors={sensors}
                                            collisionDetection={closestCenter}
                                            onDragEnd={handleDragEnd}
                                        >
                                            <SortableContext
                                                items={sortByItems}
                                                strategy={
                                                    verticalListSortingStrategy
                                                }
                                            >
                                                {sortByItems.map((id) =>
                                                    mapSortByIndexToElement(id),
                                                )}
                                            </SortableContext>
                                        </DndContext>
                                    </ul>
                                </fieldset>
                            </Form.Group>
                            <Form.Group className="mb-2">
                                <fieldset>
                                    <legend>{t("sortDirection")}</legend>
                                    <div className="form-check">
                                        <input
                                            type="radio"
                                            className="form-check-input"
                                            id="sort-direction-ascending"
                                            checked={
                                                sortDirection ===
                                                SortDirection.Ascending
                                            }
                                            onChange={() => {
                                                setSortDirection(
                                                    SortDirection.Ascending,
                                                )
                                                setSearchParams((p) => {
                                                    p.set(
                                                        "sort_direction",
                                                        SortDirection.Ascending,
                                                    )
                                                    p.set("p", "1")
                                                    return p
                                                })
                                                ;(async () =>
                                                    await search({
                                                        query: query,
                                                        start: 1,
                                                        amount: PAGE_SIZE,
                                                        sort_by:
                                                            sortByItems.map(
                                                                (i) =>
                                                                    SORT_BY_INDEX[
                                                                        i
                                                                    ],
                                                            ),
                                                        sort_direction:
                                                            SortDirection.Ascending,
                                                        packs: packs,
                                                    } satisfies HitSearchQuery))()
                                            }}
                                        />
                                        <label
                                            htmlFor="sort-direction-ascending"
                                            className="form-check-label"
                                        >
                                            {t("ascending")}
                                        </label>
                                    </div>
                                    <div className="form-check">
                                        <input
                                            type="radio"
                                            className="form-check-input"
                                            id="sort-direction-descending"
                                            checked={
                                                sortDirection ===
                                                SortDirection.Descending
                                            }
                                            onChange={() => {
                                                setSortDirection(
                                                    SortDirection.Descending,
                                                )
                                                setSearchParams((p) => {
                                                    p.set(
                                                        "sort_direction",
                                                        SortDirection.Descending,
                                                    )
                                                    p.set("p", "1")
                                                    return p
                                                })
                                                ;(async () =>
                                                    await search({
                                                        query: query,
                                                        start: 1,
                                                        amount: PAGE_SIZE,
                                                        sort_by:
                                                            sortByItems.map(
                                                                (i) =>
                                                                    SORT_BY_INDEX[
                                                                        i
                                                                    ],
                                                            ),
                                                        sort_direction:
                                                            SortDirection.Descending,
                                                        packs: packs,
                                                    } satisfies HitSearchQuery))()
                                            }}
                                        />
                                        <label
                                            htmlFor="sort-direction-descending"
                                            className="form-check-label"
                                        >
                                            {t("descending")}
                                        </label>
                                    </div>
                                </fieldset>
                            </Form.Group>
                            <Form.Group className="mb-2">
                                <Button
                                    aria-expanded={false}
                                    onClick={(e) => {
                                        e.preventDefault()
                                        setShowPackFilter(true)
                                    }}
                                >
                                    {t("pack", {
                                        count: availablePacks.length,
                                    }) +
                                        ": " +
                                        availablePacks.length +
                                        ", " +
                                        t("filtered") +
                                        ": " +
                                        packs.length}
                                </Button>
                                <PacksModal
                                    selected={packs}
                                    packs={availablePacks}
                                    show={showPackFilter}
                                    onHide={(selected) => {
                                        setPacks(selected)
                                        setShowPackFilter(false)
                                        revalidate()
                                        setSearchParams((p) => {
                                            p.delete("pack")
                                            selected.forEach((pack) =>
                                                p.append("pack", pack),
                                            )
                                            p.set("p", "1")
                                            return p
                                        })
                                        ;(async () =>
                                            await search({
                                                query: query,
                                                start: 1,
                                                amount: PAGE_SIZE,
                                                sort_by: sortByItems.map(
                                                    (i) => SORT_BY_INDEX[i],
                                                ),
                                                sort_direction: sortDirection,
                                                packs: selected,
                                            } satisfies HitSearchQuery))()
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
                    {user?.permissions.write_hits ? (
                        <>
                            <Link to="/hits/create">{t("createHit")}</Link>
                            <Button
                                onClick={async () => {
                                    const yml = await hitService.exportHits(
                                        query,
                                        packs,
                                    )
                                    const elem = document.createElement("a")
                                    elem.setAttribute(
                                        "href",
                                        `data:application/x-yaml;charset=utf-8,${encodeURIComponent(yml)}`,
                                    )
                                    elem.setAttribute("download", "hits.yml")
                                    elem.click()
                                    EventManager.publish(Events.downloadStarted)
                                }}
                            >
                                {t("exportHits")}
                            </Button>
                        </>
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
                                {canReadIssues ? <th>{t("issues")}</th> : null}
                                <th>{t("pack", { count: 2 })}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {hitResults.results.map((hit, i) => (
                                <tr key={hit.id}>
                                    <td>
                                        <Link to={"/hits/" + hit.id}>
                                            {hit.title}
                                        </Link>
                                    </td>
                                    <td>{hit.artist}</td>
                                    <td>{hit.year}</td>
                                    <td>{hit.belongs_to}</td>
                                    {canReadIssues ? (
                                        <td>{hit.issues?.length ?? 0}</td>
                                    ) : null}
                                    <td>
                                        <Button
                                            aria-expanded={false}
                                            onClick={() =>
                                                setShowPacksModal((v) => {
                                                    v[i] = true
                                                })
                                            }
                                        >
                                            {t("pack", {
                                                count: hit.packs.length,
                                            }) +
                                                ": " +
                                                hit.packs.length}
                                        </Button>
                                        <PacksModal
                                            show={showPacksModal[i]}
                                            onHide={() =>
                                                setShowPacksModal((v) => {
                                                    v[i] = false
                                                })
                                            }
                                            packs={availablePacks.filter((p) =>
                                                hit.packs.includes(p.id),
                                            )}
                                        />
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </Table>
                    <Pagination>
                        <Pagination.Item
                            key="page-first"
                            disabled={page === 1}
                            onClick={async () => {
                                setSearchParams((p) => {
                                    p.set("p", "1")
                                    return p
                                })
                                await search({
                                    start: 1,
                                    amount: PAGE_SIZE,
                                    sort_by: sortByItems.map(
                                        (i) => SORT_BY_INDEX[i],
                                    ),
                                    sort_direction: sortDirection,
                                    query: query,
                                    packs: packs,
                                } satisfies HitSearchQuery)
                            }}
                        >
                            {t("first")}
                        </Pagination.Item>
                        <Pagination.Item
                            key="page-previous"
                            disabled={page === 1}
                            onClick={async () => {
                                setSearchParams((p) => {
                                    p.set("p", `${page - 1}`)
                                    return p
                                })
                                await search({
                                    start: PAGE_SIZE * (page - 2) + 1,
                                    amount: PAGE_SIZE,
                                    sort_by: sortByItems.map(
                                        (i) => SORT_BY_INDEX[i],
                                    ),
                                    sort_direction: sortDirection,
                                    query: query,
                                    packs: packs,
                                } satisfies HitSearchQuery)
                                setPage(page - 1)
                            }}
                        >
                            {t("previous")}
                        </Pagination.Item>
                        {Array.from(
                            { length: getPageCount() },
                            (_, i) => i,
                        ).flatMap((i) => {
                            const pages = []

                            if (
                                (i < page + PAGE_RANGE &&
                                    i >= page - PAGE_RANGE - 1) ||
                                PAGE_SKIPS.includes(Math.abs(i - page + 1))
                            )
                                pages.push(
                                    <Pagination.Item
                                        key={`page-${i + 1}`}
                                        active={i + 1 === page}
                                        activeLabel={t("current")}
                                        onClick={async () => {
                                            setSearchParams((p) => {
                                                p.set("p", `${i + 1}`)
                                                return p
                                            })
                                            await search({
                                                start: PAGE_SIZE * i + 1,
                                                amount: PAGE_SIZE,
                                                sort_by: sortByItems.map(
                                                    (i) => SORT_BY_INDEX[i],
                                                ),
                                                sort_direction: sortDirection,
                                                query: query,
                                                packs: packs,
                                            } satisfies HitSearchQuery)
                                            setPage(i + 1)
                                        }}
                                    >
                                        {i + 1}
                                    </Pagination.Item>,
                                )

                            if (
                                i === page + PAGE_RANGE - 1 ||
                                PAGE_SKIPS.includes(Math.abs(i - page + 1))
                            )
                                pages.push(
                                    <Pagination.Item
                                        as="div"
                                        key={`page-div-${i + 1}`}
                                    >
                                        ...
                                    </Pagination.Item>,
                                )

                            return pages
                        })}
                        <Pagination.Item
                            key="page-next"
                            disabled={page === getPageCount()}
                            onClick={async () => {
                                setSearchParams((p) => {
                                    p.set("p", `${page + 1}`)
                                    return p
                                })
                                await search({
                                    start: PAGE_SIZE * page + 1,
                                    amount: PAGE_SIZE,
                                    sort_by: sortByItems.map(
                                        (i) => SORT_BY_INDEX[i],
                                    ),
                                    sort_direction: sortDirection,
                                    query: query,
                                    packs: packs,
                                } satisfies HitSearchQuery)
                                setPage(page + 1)
                            }}
                        >
                            {t("next")}
                        </Pagination.Item>
                        <Pagination.Item
                            key="page-last"
                            disabled={page === getPageCount()}
                            onClick={async () => {
                                setSearchParams((p) => {
                                    p.set("p", `${getPageCount()}`)
                                    return p
                                })
                                await search({
                                    start: PAGE_SIZE * (getPageCount() - 1) + 1,
                                    amount: PAGE_SIZE,
                                    sort_by: sortByItems.map(
                                        (i) => SORT_BY_INDEX[i],
                                    ),
                                    sort_direction: sortDirection,
                                    query: query,
                                    packs: packs,
                                } satisfies HitSearchQuery)
                                setPage(getPageCount())
                            }}
                        >
                            {t("last")}
                        </Pagination.Item>
                    </Pagination>
                </Col>
            </Row>
        </>
    )
}
