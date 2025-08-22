import { useEffect } from "react"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
import { useTranslation } from "react-i18next"
import { useImmer } from "use-immer"
import { Pack } from "../entities"

export default function ViewPacksModal({
    packs,
    onHide,
    show,
    selected,
}: {
    packs: Pack[]
    onHide: (selected: string[]) => void
    show: boolean
    selected?: string[]
}) {
    const [selectedPacks, setSelectedPacks] = useImmer<string[]>([])
    const { t } = useTranslation()

    useEffect(() => {
        if (selected) setSelectedPacks(selected)
    }, [selected, setSelectedPacks])

    return (
        <Modal show={show} onHide={() => onHide(selectedPacks)}>
            <Modal.Header closeButton closeLabel={t("close")} />
            <Modal.Body>
                <Form>
                    <ul>
                        {packs.map((p) =>
                            selected ? (
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
