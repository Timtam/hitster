import { Helmet } from "@dr.pogodin/react-helmet"
import classNames from "classnames"
import natsort from "natsort"
import { useEffect, useMemo, useState } from "react"
import Button from "react-bootstrap/Button"
import Form from "react-bootstrap/Form"
import { useTranslation } from "react-i18next"
import { Navigate, useLoaderData } from "react-router"
import { useImmer } from "use-immer"
import { useContext } from "../context"
import { FullHit, Pack } from "../entities"
import HitService from "../services/hits.service"
import { RE_YOUTUBE } from "../utils"

export default function CreateHit() {
    const hitService = useMemo(() => new HitService(), [])
    const sorter = useMemo(() => natsort(), [])
    const { t } = useTranslation()
    const availablePacks = useLoaderData() as Pack[]
    const { user, showError } = useContext()
    const [editingHit, setEditingHit] = useImmer<FullHit>({
        title: "",
        artist: "",
        year: 0,
        belongs_to: "",
        playback_offset: 0,
        yt_id: "",
        packs: [],
    } satisfies FullHit)
    const [youtubeUrl, setYoutubeUrl] = useState("")
    const [isUrlValid, setIsUrlValid] = useState(true)

    useEffect(() => {
        setIsUrlValid(RE_YOUTUBE.test(youtubeUrl))
    }, [youtubeUrl, setIsUrlValid])

    return !user?.permissions.can_write_hits ? (
        <Navigate to="/hits" />
    ) : (
        <>
            <Helmet>
                <title>{t("createHit") + " - Hitster"}</title>
            </Helmet>
            <h2>{t("createHit")}</h2>
            <Form onSubmit={(e) => e.preventDefault()}>
                <Form.Group className="mb-2">
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
                </Form.Group>
                <Form.Group className="mb-2">
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
                </Form.Group>
                <Form.Group className="mb-2">
                    <Form.Control
                        type="number"
                        title={t("year")}
                        value={editingHit.year}
                        onChange={(e) => {
                            let year = parseInt(e.currentTarget.value, 10)
                            setEditingHit((h) => {
                                h.year = year
                            })
                        }}
                    />
                </Form.Group>
                <Form.Group className="mb-2">
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
                </Form.Group>
                <Form.Group className="mb-2">
                    <Form.Text muted>{t("pack", { count: 2 })}</Form.Text>
                    <ul>
                        {availablePacks
                            .toSorted((a, b) => sorter(a.name, b.name))
                            .map((p) => (
                                <Form.Check
                                    type="checkbox"
                                    label={p.name}
                                    id={`pack-${p.id}`}
                                    key={`pack-${p.id}`}
                                    checked={editingHit.packs.includes(p.id)}
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
                            ))}
                    </ul>
                </Form.Group>
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
                            let text = e.currentTarget.value
                            setYoutubeUrl(text)
                            let match = RE_YOUTUBE.exec(text)
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
                            let offset = parseInt(e.currentTarget.value, 10)
                            setEditingHit((h) => {
                                h.playback_offset = offset
                            })
                        }}
                    />
                </Form.Group>
                <Button
                    disabled={!isUrlValid}
                    onClick={async () => {
                        try {
                            await hitService.createHit(editingHit)
                            setEditingHit({
                                title: "",
                                artist: "",
                                year: 0,
                                belongs_to: "",
                                playback_offset: 0,
                                yt_id: "",
                                packs: editingHit.packs,
                            } satisfies FullHit)
                            setYoutubeUrl("")
                        } catch (e) {
                            showError((e as any).message)
                        }
                    }}
                >
                    {t("save")}
                </Button>
            </Form>
        </>
    )
}
