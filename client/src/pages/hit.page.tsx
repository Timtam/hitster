import { Helmet } from "@dr.pogodin/react-helmet"
import natsort from "natsort"
import { useMemo } from "react"
import Form from "react-bootstrap/Form"
import { useTranslation } from "react-i18next"
import { useLoaderData } from "react-router"
import YouTube from "react-youtube"
import { FullHit, Pack } from "../entities"

export default function Hit() {
    const sorter = useMemo(() => natsort(), [])
    const { t } = useTranslation()
    const [hit, availablePacks] = useLoaderData() as [FullHit, Pack[]]

    return (
        <>
            <Helmet>
                <title>{`${hit.artist}: ${hit.title} - Hitster`}</title>
            </Helmet>
            <h2>{`${hit.artist}: ${hit.title}`}</h2>
            <Form onSubmit={(e) => e.preventDefault()}>
                <Form.Group className="mb-2">
                    <Form.Text muted>{t("title") + ": " + hit.title}</Form.Text>
                </Form.Group>
                <Form.Group className="mb-2">
                    <Form.Text muted>
                        {t("artist") + ": " + hit.artist}
                    </Form.Text>
                </Form.Group>
                <Form.Group className="mb-2">
                    <Form.Text muted>{t("year") + ": " + hit.year}</Form.Text>
                </Form.Group>
                <Form.Group className="mb-2">
                    <Form.Text muted>
                        {t("belongsTo") +
                            ": " +
                            (hit.belongs_to ? hit.belongs_to : "---")}
                    </Form.Text>
                </Form.Group>
                <Form.Group className="mb-2">
                    <Form.Text muted>{t("pack", { count: 2 })}</Form.Text>
                    <ul>
                        {availablePacks
                            .filter((p) => hit.packs.includes(p.id))
                            .toSorted((a, b) => sorter(a.name, b.name))
                            .map((p) => (
                                <li>{p.name}</li>
                            ))}
                    </ul>
                </Form.Group>
                <YouTube
                    videoId={hit.yt_id}
                    opts={{
                        playerVars: { start: hit.playback_offset, autoplay: 0 },
                    }}
                />
            </Form>
        </>
    )
}
