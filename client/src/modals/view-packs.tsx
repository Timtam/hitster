import { useEffect, useMemo } from "react"
import Button from "react-bootstrap/Button"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
import { useTranslation } from "react-i18next"
import { useImmer } from "use-immer"
import { useContext } from "../context"
import { Pack } from "../entities"
import HitService from "../services/hits.service"

export default function ViewPacksModal({
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
    const hitService = useMemo(() => new HitService(), [])
    const [packs, setPacks] = useImmer<Pack[]>([])
    const [selectedPacks, setSelectedPacks] = useImmer<string[]>([])
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
                <Form>
                    <ul>
                        {packs.map((p) =>
                            selected ? (
                                <>
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
                                        checked={selectedPacks.includes(p.id)}
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
                                </>
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
