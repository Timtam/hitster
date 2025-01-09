import i18n from "i18next"
import LanguageDetector from "i18next-browser-languagedetector"
import { initReactI18next } from "react-i18next"
import de from "./locale/de.yml"
import en from "./locale/en.yml"

i18n.use(LanguageDetector)
    .use(initReactI18next)
    .init({
        fallbackLng: "en",
        interpolation: {
            escapeValue: false,
        },
        resources: {
            en: { translation: { ...en } },
            de: { translation: { ...de } },
        }, // Where we're gonna put translations' files
    })
