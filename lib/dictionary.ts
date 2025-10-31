import type en from '@/messages/en.json'

export type Dictionary = typeof en

// Top-level section types
export type HeroDictionary = Dictionary['hero']
export type AboutDictionary = Dictionary['about']
export type SkillsDictionary = Dictionary['skills']
export type ExperienceDictionary = Dictionary['experience']
export type ProjectsDictionary = Dictionary['projects']
export type TestimonialsDictionary = Dictionary['testimonials']
export type CommandPaletteDictionary = Dictionary['commandPalette']
export type ContactDictionary = Dictionary['contact']
export type ImprintDictionary = Dictionary['imprint']
export type CookiesDictionary = Dictionary['cookies']
