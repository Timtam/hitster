import { useEffect, useRef, useState } from "react"
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
import type { Game } from "../../entities"
import { GameSettings } from "../../entities"
import GameService from "../../services/games.service"
import HitService from "../../services/hits.service"

export default ({
    game,
    show,
    onHide,
}: {
    game: Game
    show: boolean
    onHide: () => void
}) => {
    let { t } = useTranslation()
    let [goal, setGoal] = useState(0)
    let [startTokens, setStartTokens] = useState(0)
    let [hitDuration, setHitDuration] = useState(0)
    let [availablePacks, setAvailablePacks] = useState<Record<string, number>>(
        {},
    )
    let [packs, setPacks] = useState<string[]>([])
    let [rememberHits, setRememberHits] = useState(true)
    let selectAllPacks = useRef<
        (HTMLInputElement & BsPrefixRefForwardingComponent<"input", any>) | null
    >(null)
    let { showError } = useContext()

    useEffect(() => {
        if (!show) {
            setGoal(game.goal)
            setStartTokens(game.start_tokens)
            setHitDuration(game.hit_duration)
            setPacks(game.packs)
            setRememberHits(game.remember_hits)
        }
    }, [game, show])

    useEffect(() => {
        if (show) {
            ;(async () => {
                let hs = new HitService()
                let availablePacks = await hs.getAllPacks()
                setAvailablePacks(availablePacks)
                if (packs.length === 0) setPacks(Object.keys(availablePacks))
            })()
        } else {
            setAvailablePacks({})
        }
    }, [show])

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
                                        <p>{t("gameSettingsRememberHits")}</p>
                                        <Form.Group className="mb-2">
                                            <Form.Label htmlFor="checkbox-remember-hits">
                                                {t("rememberHits")}
                                            </Form.Label>
                                            <Form.Check
                                                id="checkbox-remember-hits"
                                                type="checkbox"
                                                placeholder={t("rememberHits")}
                                                checked={rememberHits}
                                                onChange={() => {
                                                    setRememberHits(
                                                        !rememberHits,
                                                    )
                                                }}
                                            />
                                        </Form.Group>
                                    </Form>
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
                            <Row className="text-center">
                                <Col className="mx-auto">
                                    <Form.Group className="mb-2">
                                        <Form.Label htmlFor="checkbox-select-all-packs">
                                            {t("selectAll")}
                                        </Form.Label>
                                        <Form.Check
                                            ref={selectAllPacks}
                                            id="checkbox-select-all-packs"
                                            type="checkbox"
                                            placeholder={t("selectAll")}
                                            onChange={(e) => {
                                                if (e.currentTarget.checked)
                                                    setPacks(
                                                        Object.keys(
                                                            availablePacks,
                                                        ),
                                                    )
                                                else setPacks([])
                                            }}
                                        />
                                    </Form.Group>
                                    <hr />
                                    <ToggleButtonGroup
                                        vertical
                                        type="checkbox"
                                        value={packs}
                                        onChange={(e) => setPacks(e)}
                                    >
                                        {Object.keys(availablePacks)
                                            .toSorted()
                                            .map((p) => (
                                                <ToggleButton
                                                    className="me-2 mb-2"
                                                    value={p}
                                                    id={`pack-${slugify(p)}`}
                                                    key={`pack-${slugify(p)}`}
                                                >
                                                    {`${p} (${availablePacks[p]} ${t("hit", { count: 2 })})`}
                                                </ToggleButton>
                                            ))}
                                    </ToggleButtonGroup>
                                </Col>
                            </Row>
                            <Row>
                                <Col>
                                    <Button
                                        onClick={async () => {
                                            let gs = new GameService()
                                            try {
                                                await gs.update(
                                                    game.id,
                                                    GameSettings.parse({
                                                        goal: goal,
                                                        hit_duration:
                                                            hitDuration,
                                                        start_tokens:
                                                            startTokens,
                                                        packs: packs,
                                                        remember_hits:
                                                            rememberHits,
                                                    }),
                                                )
                                                onHide()
                                            } catch (e) {
                                                showError((e as any).message)
                                            }
                                        }}
                                    >
                                        {t("save")}
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
