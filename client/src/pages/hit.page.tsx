import { Helmet } from "@dr.pogodin/react-helmet"
import EventManager from "@lomray/event-manager"
import classNames from "classnames"
import deepcopy from "deep-copy"
import natsort from "natsort"
import { useEffect, useMemo, useState } from "react"
import Button from "react-bootstrap/Button"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
import Table from "react-bootstrap/Table"
import { useTranslation } from "react-i18next"
import { Link, useLoaderData, useNavigate } from "react-router"
import YouTube from "react-youtube"
import { useImmer } from "use-immer"
import { useContext } from "../context"
import { FullHit, HitIssueType, Pack } from "../entities"
import { Events, IssueCreatedData, IssueDeletedData } from "../events"
import FA from "../focus-anchor"
import { useRevalidate } from "../hooks"
import ReportHitIssueModal from "../modals/report-hit-issue"
import HitService from "../services/hits.service"
import { RE_YOUTUBE } from "../utils"

function DeleteHitModal({
    show,
    onHide,
}: {
    show: boolean
    onHide: (yes: boolean) => void
}) {
    const { t } = useTranslation()

    useEffect(() => {
        if (show) EventManager.publish(Events.popup)
    }, [show])

    return (
        <Modal show={show} onHide={() => {}}>
            <Modal.Header>
                <Modal.Title>{t("deleteHit")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                {show ? (
                    <>
                        <h2>{t("deleteHitQuestion")}</h2>
                        <Button onClick={() => onHide(false)}>{t("no")}</Button>
                        <Button onClick={() => onHide(true)}>{t("yes")}</Button>
                    </>
                ) : (
                    ""
                )}
            </Modal.Body>
        </Modal>
    )
}

function DeleteIssueModal({
    show,
    onHide,
}: {
    show: boolean
    onHide: (yes: boolean) => void
}) {
    const { t } = useTranslation()

    useEffect(() => {
        if (show) EventManager.publish(Events.popup)
    }, [show])

    return (
        <Modal show={show} onHide={() => {}}>
            <Modal.Header>
                <Modal.Title>{t("deleteIssue")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                {show ? (
                    <>
                        <h2>{t("deleteIssueQuestion")}</h2>
                        <Button onClick={() => onHide(false)}>{t("no")}</Button>
                        <Button onClick={() => onHide(true)}>{t("yes")}</Button>
                    </>
                ) : (
                    ""
                )}
            </Modal.Body>
        </Modal>
    )
}

export default function Hit() {
    const hitService = useMemo(() => new HitService(), [])
    const sorter = useMemo(() => natsort(), [])
    const { t } = useTranslation()
    const [hit, availablePacks] = useLoaderData() as [FullHit, Pack[]]
    const { user, showError } = useContext()
    const [editing, setEditing] = useState(false)
    const [editingHit, setEditingHit] = useImmer<FullHit>({
        title: "",
        artist: "",
        year: 0,
        belongs_to: "",
        playback_offset: 0,
        yt_id: "",
        packs: [],
        id: "",
    } satisfies FullHit)
    const [youtubeUrl, setYoutubeUrl] = useState("")
    const [isUrlValid, setIsUrlValid] = useState(true)
    const [showDeleteHitModal, setShowDeleteHitModal] = useState(false)
    const [showReportIssueModal, setShowReportIssueModal] = useState(false)
    const [issueToDelete, setIssueToDelete] = useState<string | undefined>()
    const reload = useRevalidate()
    const navigate = useNavigate()
    const canDeleteIssues = useMemo(
        () => user?.permissions.delete_issues === true,
        [user],
    )
    const canReadIssues = useMemo(
        () => user?.permissions.read_issues === true,
        [user],
    )

    useEffect(() => {
        setIsUrlValid(RE_YOUTUBE.test(youtubeUrl))
    }, [youtubeUrl, setIsUrlValid])

    useEffect(() => {
        if (!canReadIssues || !hit.id) return

        const unsubscribeIssueCreated = EventManager.subscribe(
            Events.issueCreated,
            (e: IssueCreatedData) => {
                if (e.issue.hit_id === hit.id) reload()
            },
        )
        const unsubscribeIssueDeleted = EventManager.subscribe(
            Events.issueDeleted,
            (e: IssueDeletedData) => {
                if (e.hitId === hit.id) reload()
            },
        )

        return () => {
            unsubscribeIssueCreated()
            unsubscribeIssueDeleted()
        }
    }, [canReadIssues, hit.id, reload])

    return (
        <>
            <Helmet>
                <title>{`${hit.artist}: ${hit.title} - Hitster`}</title>
            </Helmet>
            <FA>
                <h2>{`${hit.artist}: ${hit.title}`}</h2>
            </FA>
            {!editing && user?.permissions.write_hits ? (
                <>
                    <Button
                        onClick={() => {
                            setEditingHit(deepcopy(hit))
                            setYoutubeUrl(
                                `https://www.youtube.com/watch?v=${hit.yt_id}`,
                            )
                            setEditing(true)
                        }}
                    >
                        {t("edit")}
                    </Button>
                    <Button onClick={() => setShowDeleteHitModal(true)}>
                        {t("delete")}
                    </Button>
                    <DeleteHitModal
                        show={showDeleteHitModal}
                        onHide={(yes) => {
                            setShowDeleteHitModal(false)
                            if (yes) {
                                ;(async () => {
                                    try {
                                        await hitService.deleteHit(hit.id!)
                                        navigate(-1)
                                    } catch (e) {
                                        showError((e as any).message)
                                    }
                                })()
                            }
                        }}
                    />
                </>
            ) : editing ? (
                <Button
                    onClick={() => {
                        setEditing(false)
                        setEditingHit((h) => {
                            h.title = ""
                            h.artist = ""
                            h.year = 0
                            h.playback_offset = 0
                            h.id = ""
                            h.yt_id = ""
                            h.packs = []
                            h.belongs_to = ""
                        })
                        setYoutubeUrl("")
                    }}
                >
                    {t("cancel")}
                </Button>
            ) : (
                ""
            )}
            {user?.permissions.write_issues ? (
                <>
                    <Button
                        className={user?.permissions.write_hits ? "ms-2" : ""}
                        onClick={() => setShowReportIssueModal(true)}
                    >
                        {t("reportIssue")}
                    </Button>
                    <ReportHitIssueModal
                        show={showReportIssueModal}
                        hitId={hit.id ?? ""}
                        onHide={() => setShowReportIssueModal(false)}
                    />
                </>
            ) : (
                ""
            )}
            <DeleteIssueModal
                show={issueToDelete !== undefined}
                onHide={(yes) => {
                    const issueId = issueToDelete
                    if (yes && issueId) {
                        ;(async () => {
                            try {
                                if (!hit.id) {
                                    showError(t("error"))
                                    setIssueToDelete(undefined)
                                    return
                                }
                                await hitService.deleteIssue(hit.id, issueId)
                                reload()
                            } catch (e) {
                                showError((e as any).message)
                            }
                            setIssueToDelete(undefined)
                        })()
                    } else {
                        setIssueToDelete(undefined)
                    }
                }}
            />
            <Form onSubmit={(e) => e.preventDefault()}>
                <Form.Group className="mb-2">
                    {editing ? (
                        <Form.Control
                            type="input"
                            placeholder={t("title")}
                            value={editingHit.title}
                            onChange={(e) => {
                                const title = e.currentTarget.value
                                setEditingHit((h) => {
                                    h.title = title
                                })
                            }}
                        />
                    ) : (
                        <Form.Text muted>
                            {t("title") + ": " + hit.title}
                        </Form.Text>
                    )}
                </Form.Group>
                <Form.Group className="mb-2">
                    {editing ? (
                        <Form.Control
                            type="input"
                            placeholder={t("artist")}
                            value={editingHit.artist}
                            onChange={(e) => {
                                const artist = e.currentTarget.value
                                setEditingHit((h) => {
                                    h.artist = artist
                                })
                            }}
                        />
                    ) : (
                        <Form.Text muted>
                            {t("artist") + ": " + hit.artist}
                        </Form.Text>
                    )}
                </Form.Group>
                <Form.Group className="mb-2">
                    {editing ? (
                        <Form.Control
                            type="number"
                            title={t("year")}
                            value={editingHit.year}
                            onChange={(e) => {
                                const year = parseInt(e.currentTarget.value, 10)
                                setEditingHit((h) => {
                                    h.year = year
                                })
                            }}
                        />
                    ) : (
                        <Form.Text muted>
                            {t("year") + ": " + hit.year}
                        </Form.Text>
                    )}
                </Form.Group>
                <Form.Group className="mb-2">
                    {editing ? (
                        <Form.Control
                            type="input"
                            placeholder={t("belongsTo")}
                            value={editingHit.belongs_to}
                            onChange={(e) => {
                                const belongsTo = e.currentTarget.value
                                setEditingHit((h) => {
                                    h.belongs_to = belongsTo
                                })
                            }}
                        />
                    ) : (
                        <Form.Text muted>
                            {t("belongsTo") +
                                ": " +
                                (hit.belongs_to ? hit.belongs_to : "---")}
                        </Form.Text>
                    )}
                </Form.Group>
                <Form.Group className="mb-2">
                    <Form.Text muted>{t("pack", { count: 2 })}</Form.Text>
                    <ul>
                        {editing
                            ? availablePacks
                                  .toSorted((a, b) => sorter(a.name, b.name))
                                  .map((p) => (
                                      <Form.Check
                                          type="checkbox"
                                          label={p.name}
                                          id={`pack-${p.id}`}
                                          key={`pack-${p.id}`}
                                          checked={editingHit.packs.includes(
                                              p.id,
                                          )}
                                          onChange={() =>
                                              setEditingHit((h) => {
                                                  if (h.packs.includes(p.id))
                                                      h.packs.splice(
                                                          h.packs.indexOf(p.id),
                                                          1,
                                                      )
                                                  else h.packs.push(p.id)
                                              })
                                          }
                                      />
                                  ))
                            : hit.packs
                                  .map(
                                      (p) =>
                                          availablePacks.find(
                                              (pp) => pp.id === p,
                                          )!,
                                  )
                                  .toSorted((a, b) => sorter(a.name, b.name))
                                  .map((p) => (
                                      <li key={`pack-${p.id}`}>
                                          <Link
                                              to={`/hits/?pack=${p.id}`}
                                          >{`${p.name}`}</Link>
                                      </li>
                                  ))}
                    </ul>
                </Form.Group>
                {editing ? (
                    <Form.Group className="mb-2">
                        <Form.Control
                            type="input"
                            placeholder={t("youtubeUrl")}
                            isInvalid={!isUrlValid}
                            aria-invalid={!isUrlValid}
                            aria-errormessage={
                                !isUrlValid ? "error-invalid-url" : ""
                            }
                            value={youtubeUrl}
                            onChange={(e) => {
                                const text = e.currentTarget.value
                                setYoutubeUrl(text)
                                const match = RE_YOUTUBE.exec(text)
                                if (match !== null)
                                    setEditingHit((h) => {
                                        h.yt_id = match[1]
                                    })
                            }}
                        />
                        <Form.Text
                            aria-hidden={isUrlValid}
                            className={classNames({
                                "visually-hidden": isUrlValid,
                            })}
                            muted
                            id="error-invalid-url"
                        >
                            {t("youtubeUrlInvalid")}
                        </Form.Text>
                        <Form.Control
                            type="number"
                            title={t("playbackOffset")}
                            min={0}
                            value={editingHit.playback_offset}
                            onChange={(e) => {
                                const offset = parseInt(
                                    e.currentTarget.value,
                                    10,
                                )
                                setEditingHit((h) => {
                                    h.playback_offset = offset
                                })
                            }}
                        />
                    </Form.Group>
                ) : (
                    <YouTube
                        videoId={hit.yt_id}
                        opts={{
                            playerVars: {
                                start: hit.playback_offset,
                                autoplay: 0,
                            },
                        }}
                    />
                )}
                {editing ? (
                    <Button
                        disabled={!isUrlValid}
                        onClick={async () => {
                            try {
                                await hitService.updateHit(editingHit)
                                setEditing(false)
                                setEditingHit({
                                    title: "",
                                    artist: "",
                                    year: 0,
                                    belongs_to: "",
                                    playback_offset: 0,
                                    yt_id: "",
                                    packs: [],
                                    id: "",
                                } satisfies FullHit)
                                reload()
                            } catch (e) {
                                showError((e as any).message)
                            }
                        }}
                    >
                        {t("save")}
                    </Button>
                ) : (
                    ""
                )}
            </Form>
            {canReadIssues ? (
                <>
                    <h3>
                        {t("issuesHeading", {
                            count: hit.issues ? hit.issues.length : 0,
                        })}
                    </h3>
                    <Table responsive>
                        <thead>
                            <tr>
                                <th>{t("type")}</th>
                                <th>{t("message")}</th>
                                <th>{t("createdAt")}</th>
                                <th>{t("lastModified")}</th>
                                {canDeleteIssues ? (
                                    <th>{t("action")}</th>
                                ) : null}
                            </tr>
                        </thead>
                        <tbody>
                            {hit.issues && hit.issues.length > 0 ? (
                                hit.issues.map((issue) => (
                                    <tr key={issue.id}>
                                        <td>
                                            {issue.type === HitIssueType.Auto
                                                ? t("automatic")
                                                : t("custom")}
                                        </td>
                                        <td>{issue.message}</td>
                                        <td>
                                            {issue.created_at.toLocaleString()}
                                        </td>
                                        <td>
                                            {issue.last_modified.toLocaleString()}
                                        </td>
                                        {canDeleteIssues ? (
                                            <td>
                                                <Button
                                                    onClick={() =>
                                                        setIssueToDelete(
                                                            issue.id,
                                                        )
                                                    }
                                                >
                                                    {t("delete")}
                                                </Button>
                                            </td>
                                        ) : null}
                                    </tr>
                                ))
                            ) : (
                                <tr>
                                    <td colSpan={canDeleteIssues ? 5 : 4}>
                                        {t("noIssues")}
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </Table>
                </>
            ) : (
                ""
            )}
        </>
    )
}
