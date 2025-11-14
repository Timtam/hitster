import natsort from "natsort"
import pickRandom from "pick-random"
import { useEffect, useMemo, useRef, useState } from "react"
import Button from "react-bootstrap/Button"
import Col from "react-bootstrap/Col"
import Form from "react-bootstrap/Form"
import Modal from "react-bootstrap/Modal"
import Row from "react-bootstrap/Row"
import Spinner from "react-bootstrap/Spinner"
import ToggleButton from "react-bootstrap/ToggleButton"
import ToggleButtonGroup from "react-bootstrap/ToggleButtonGroup"
import { BsPrefixRefForwardingComponent } from "react-bootstrap/helpers"
import { useTranslation } from "react-i18next"
import slugify from "slugify"
import { useContext } from "../../context"
import type { Game, Pack } from "../../entities"
import { GameSettings as GameSettingsEntity, GameState } from "../../entities"
import GameService from "../../services/games.service"
import HitService from "../../services/hits.service"

export default function GameSettings({
    game,
    editable,
    show,
    onHide,
}: {
    game: Game
    editable: boolean
    show: boolean
    onHide: () => void
}) {
    const sorter = useMemo(() => natsort(), [])
    const { t } = useTranslation()
    const [goal, setGoal] = useState(0)
    const [startTokens, setStartTokens] = useState(0)
    const [hitDuration, setHitDuration] = useState(0)
    const [availablePacks, setAvailablePacks] = useState<Pack[]>([])
    const [packs, setPacks] = useState<string[]>([])
    const selectAllPacks = useRef<
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (HTMLInputElement & BsPrefixRefForwardingComponent<"input", any>) | null
    >(null)
    const { showError } = useContext()
    const [randomPacks, setRandomPacks] = useState(5)

    useEffect(() => {
        if (!show) {
            setGoal(game.goal)
            setStartTokens(game.start_tokens)
            setHitDuration(game.hit_duration)
            setPacks(game.packs)
        }
    }, [game, show])

    useEffect(() => {
        if (show) {
            ;(async () => {
                const hs = new HitService()
                const availablePacks = await hs.getAllPacks()
                setAvailablePacks(availablePacks)
                if (packs.length === 0)
                    setPacks(availablePacks.map((p) => p.id))
            })()
        } else {
            setAvailablePacks([])
        }
        // don't do this at home kids
        // we're waiting for useEffectEvent to become stable
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [setAvailablePacks, setPacks, show])

    useEffect(() => {
        if (selectAllPacks.current === null) return
        if (
            packs.length > 0 &&
            packs.length !== Object.keys(availablePacks).length
        ) {
            selectAllPacks.current.checked = false
            selectAllPacks.current.indeterminate = true
        } else if (packs.length === 0) {
            selectAllPacks.current.checked = false
            selectAllPacks.current.indeterminate = false
        } else if (packs.length === Object.keys(availablePacks).length) {
            selectAllPacks.current.checked = true
            selectAllPacks.current.indeterminate = false
        }
    }, [packs, availablePacks])

    return (
        <Modal show={show} onHide={onHide}>
            <Modal.Header closeButton closeLabel={t("cancel")}>
                <Modal.Title>{t("gameSettings")}</Modal.Title>
            </Modal.Header>
            <Modal.Body>
                {show ? (
                    Object.keys(availablePacks).length === 0 ? (
                        <Spinner animation="border" role="status">
                            <span className="visually-hidden">
                                {t("loading")}
                            </span>
                        </Spinner>
                    ) : (
                        <>
                            <Row>
                                <Col>
                                    <h2 className="h4">{t("gameSettings")}</h2>
                                    <Form>
                                        <p>{t("gameSettingsHitGoal")}</p>
                                        <Form.Group className="mb-2">
                                            <Form.Label>{t("goal")}</Form.Label>
                                            <Form.Control
                                                type="number"
                                                min={1}
                                                placeholder={t("goal")}
                                                disabled={!editable}
                                                value={goal}
                                                onChange={(e) =>
                                                    setGoal(
                                                        e.currentTarget
                                                            .value === ""
                                                            ? 1
                                                            : parseInt(
                                                                  e
                                                                      .currentTarget
                                                                      .value,
                                                                  10,
                                                              ),
                                                    )
                                                }
                                            />
                                        </Form.Group>
                                        <p>{t("gameSettingsStartTokens")}</p>
                                        <Form.Group className="mb-2">
                                            <Form.Label>
                                                {t("startTokens")}
                                            </Form.Label>
                                            <Form.Control
                                                type="number"
                                                min={0}
                                                placeholder={t("startTokens")}
                                                value={startTokens}
                                                disabled={!editable}
                                                onChange={(e) =>
                                                    setStartTokens(
                                                        e.currentTarget
                                                            .value === ""
                                                            ? 0
                                                            : parseInt(
                                                                  e
                                                                      .currentTarget
                                                                      .value,
                                                                  10,
                                                              ),
                                                    )
                                                }
                                            />
                                        </Form.Group>
                                        <p>{t("gameSettingsHitDuration")}</p>
                                        <Form.Group className="mb-2">
                                            <Form.Label>
                                                {t("hitDuration")}
                                            </Form.Label>
                                            <Form.Control
                                                type="number"
                                                min={1}
                                                placeholder={t("hitDuration")}
                                                value={hitDuration}
                                                disabled={!editable}
                                                onChange={(e) =>
                                                    setHitDuration(
                                                        e.currentTarget
                                                            .value === ""
                                                            ? 1
                                                            : parseInt(
                                                                  e
                                                                      .currentTarget
                                                                      .value,
                                                                  10,
                                                              ),
                                                    )
                                                }
                                            />
                                        </Form.Group>
                                    </Form>
                                    <hr />
                                </Col>
                            </Row>
                            <Row>
                                <Col>
                                    <h2 className="h4">
                                        {t("pack", { count: 2 })}
                                    </h2>
                                    <p>{t("gameSettingsPacks")}</p>
                                </Col>
                            </Row>
                            <Row>
                                <Col>
                                    <Form.Group className="mb-2">
                                        <Form.Label htmlFor="checkbox-select-all-packs">
                                            {t("selectAll")}
                                        </Form.Label>
                                        <Form.Check
                                            ref={selectAllPacks}
                                            id="checkbox-select-all-packs"
                                            type="checkbox"
                                            placeholder={t("selectAll")}
                                            disabled={!editable}
                                            onChange={(e) => {
                                                if (e.currentTarget.checked)
                                                    setPacks(
                                                        availablePacks.map(
                                                            (p) => p.id,
                                                        ),
                                                    )
                                                else setPacks([])
                                            }}
                                        />
                                    </Form.Group>
                                    <Form.Group className="mb-2">
                                        <Form.Label>
                                            {t("randomPacksLabel")}
                                        </Form.Label>
                                        <Form.Control
                                            className="mb-2"
                                            type="number"
                                            min={1}
                                            max={availablePacks.length}
                                            placeholder={t("randomPacksLabel")}
                                            value={randomPacks}
                                            disabled={!editable}
                                            onChange={(e) =>
                                                setRandomPacks(
                                                    e.currentTarget.value === ""
                                                        ? 0
                                                        : parseInt(
                                                              e.currentTarget
                                                                  .value,
                                                              10,
                                                          ),
                                                )
                                            }
                                        />
                                        <Button
                                            disabled={!editable}
                                            onClick={() =>
                                                setPacks(
                                                    pickRandom(availablePacks, {
                                                        count: randomPacks,
                                                    }).map((p) => p.id),
                                                )
                                            }
                                        >
                                            {t("select")}
                                        </Button>
                                    </Form.Group>
                                    <hr />
                                    <ToggleButtonGroup
                                        vertical
                                        type="checkbox"
                                        value={packs}
                                        onChange={(e) => setPacks(e)}
                                    >
                                        {availablePacks
                                            .toSorted((a, b) =>
                                                sorter(a.name, b.name),
                                            )
                                            .map((p) => (
                                                <ToggleButton
                                                    className="me-2 mb-2"
                                                    value={p.id}
                                                    id={`pack-${slugify(p.name)}`}
                                                    key={`pack-${slugify(p.name)}`}
                                                    disabled={!editable}
                                                >
                                                    {`${p.name} (${p.hits} ${t("hit", { count: 2 })})`}
                                                </ToggleButton>
                                            ))}
                                    </ToggleButtonGroup>
                                </Col>
                            </Row>
                            <Row>
                                <Col>
                                    <Button
                                        disabled={!editable}
                                        onClick={async () => {
                                            const gs = new GameService()
                                            try {
                                                await gs.update(
                                                    game.id,
                                                    GameSettingsEntity.parse({
                                                        goal: goal,
                                                        hit_duration:
                                                            hitDuration,
                                                        start_tokens:
                                                            startTokens,
                                                        packs: packs,
                                                    }),
                                                )
                                                onHide()
                                            } catch (e) {
                                                showError(
                                                    (
                                                        e as {
                                                            message: string
                                                            status: number
                                                        }
                                                    ).message,
                                                )
                                            }
                                        }}
                                    >
                                        {game.state !== GameState.Open
                                            ? t("gameAlreadyRunning")
                                            : !editable
                                              ? t("gameSettingsNotCreator")
                                              : t("save")}
                                    </Button>
                                </Col>
                            </Row>
                        </>
                    )
                ) : (
                    ""
                )}
            </Modal.Body>
        </Modal>
    )
}
