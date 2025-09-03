import natsort from "natsort"
import { useEffect, useMemo, useState } from "react"
import Button from "react-bootstrap/Button"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
import { useTranslation } from "react-i18next"
import { useImmer } from "use-immer"
import { useContext } from "../../context"
import { Pack } from "../../entities"
import HitService from "../../services/hits.service"

function CreatePackModal({
    show,
    onHide,
}: {
    show: boolean
    onHide: (pack?: Pack) => void
}) {
    const hitService = useMemo(() => new HitService(), [])
    const { t } = useTranslation()
    const [name, setName] = useState("")
    const { showError } = useContext()

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("cancel")}>
                <Modal.Title>{t("createPack")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                <h2 className="h4">{t("createPack")}</h2>
                <Form>
                    <Form.Group
                        className="mb-2"
                        controlId="formLocalPlayerName"
                    >
                        <Form.Label>{t("pack", { count: 1 })}</Form.Label>
                        <Form.Control
                            type="input"
                            placeholder={t("name")}
                            value={name}
                            onChange={(e) => setName(e.currentTarget.value)}
                        />
                    </Form.Group>
                    <Button
                        type="submit"
                        variant="primary"
                        disabled={name === ""}
                        onClick={async (e) => {
                            e.preventDefault()
                            try {
                                const pack = await hitService.createPack(name)
                                onHide(pack)
                                setName("")
                            } catch (e) {
                                showError((e as any).message)
                            }
                        }}
                    >
                        {t("createPack")}
                    </Button>
                </Form>
            </Modal.Body>
        </Modal>
    )
}

export default function PacksModal({
    packs: initialPacks,
    onHide,
    show,
    selected,
}: {
    packs: Pack[]
    onHide: (selected: string[]) => void
    show: boolean
    selected?: string[]
}) {
    const sorter = useMemo(() => natsort(), [])
    const hitService = useMemo(() => new HitService(), [])
    const [packs, setPacks] = useImmer<Pack[]>([])
    const [selectedPacks, setSelectedPacks] = useImmer<string[]>([])
    const [showCreatePackModal, setShowCreatePackModal] = useState(false)
    const { t } = useTranslation()
    const { user, showError } = useContext()

    useEffect(() => {
        if (selected) setSelectedPacks(selected)
        if (initialPacks.length > 0) setPacks(Array.from(initialPacks))
    }, [selected, setSelectedPacks, initialPacks, setPacks])

    return (
        <Modal show={show} onHide={() => onHide(selectedPacks)}>
            <Modal.Header closeButton closeLabel={t("close")} />
            <Modal.Body>
                {selected && user?.permissions.can_write_packs ? (
                    <>
                        <Button
                            aria-expanded={false}
                            onClick={() => setShowCreatePackModal(true)}
                        >
                            {t("createPack")}
                        </Button>
                        <CreatePackModal
                            show={showCreatePackModal}
                            onHide={(pack) => {
                                if (pack)
                                    setPacks((packs) => {
                                        packs.push(pack)
                                    })
                                setShowCreatePackModal(false)
                            }}
                        />
                    </>
                ) : (
                    ""
                )}
                <Form>
                    <ul>
                        {packs
                            .toSorted((a, b) => sorter(a.name, b.name))
                            .map((p) =>
                                selected ? (
                                    <div role="listitem">
                                        <Form.Check
                                            type="checkbox"
                                            label={
                                                p.name +
                                                " (" +
                                                p.hits +
                                                " " +
                                                t("hit", { count: 2 }) +
                                                ")"
                                            }
                                            id={`pack-${p.id}`}
                                            key={`pack-${p.id}`}
                                            checked={selectedPacks.includes(
                                                p.id,
                                            )}
                                            onChange={() =>
                                                setSelectedPacks((packs) => {
                                                    if (packs.includes(p.id))
                                                        packs.splice(
                                                            packs.indexOf(p.id),
                                                            1,
                                                        )
                                                    else packs.push(p.id)
                                                })
                                            }
                                        />
                                        {user?.permissions.can_write_packs ? (
                                            <Button
                                                onClick={async () => {
                                                    try {
                                                        await hitService.deletePack(
                                                            p.id,
                                                        )
                                                        setPacks((packs) => {
                                                            packs.splice(
                                                                packs.findIndex(
                                                                    (pack) =>
                                                                        pack.id ===
                                                                        p.id,
                                                                ),
                                                                1,
                                                            )
                                                        })
                                                    } catch (e) {
                                                        showError(
                                                            (e as any).message,
                                                        )
                                                    }
                                                }}
                                            >
                                                {t("delete")}
                                            </Button>
                                        ) : (
                                            ""
                                        )}
                                    </div>
                                ) : (
                                    <li key={`pack-${p.id}`}>{`${p.name}`}</li>
                                ),
                            )}
                    </ul>
                </Form>
            </Modal.Body>
        </Modal>
    )
}
