import { describe, expect, it } from 'vitest'

import { parseResumeData } from '@/lib/resume-utilities'
import type {
  ResumeData,
  ResumeEducation,
  ResumeExperience,
  ResumePersonalInfo,
  ResumeProject,
  ResumeSkills,
} from '@/types/resume-types'

describe('resume-utilities', () => {
  describe('parseResumeData', () => {
    const validPersonalInfo: ResumePersonalInfo = {
      email: 'john@example.com',
      github: 'https://github.com/johndoe',
      linkedin: 'https://linkedin.com/in/johndoe',
      location: 'San Francisco, CA',
      name: 'John Doe',
      title: 'Senior Software Engineer',
    }

    const validSkills: ResumeSkills = {
      expertise: ['JavaScript', 'TypeScript', 'React'],
      learning: ['Rust', 'Go'],
      tools: ['Git', 'Docker', 'VS Code'],
    }

    const validExperience: readonly ResumeExperience[] = [
      {
        achievements: [
          'Built scalable web applications',
          'Improved performance by 40%',
        ],
        company: 'Tech Corp',
        endDate: 'Present',
        location: 'San Francisco, CA',
        startDate: '2020',
        title: 'Senior Developer',
      },
    ]

    const validProjects: readonly ResumeProject[] = [
      {
        description: 'A cool project',
        name: 'Project Alpha',
        technologies: ['React', 'Node.js'],
        url: 'https://github.com/johndoe/project-alpha',
      },
    ]

    const validEducation: readonly ResumeEducation[] = [
      {
        degree: 'B.S. Computer Science',
        institution: 'University of California',
        year: '2016',
      },
    ]

    const validResumeData: ResumeData = {
      education: validEducation,
      experience: validExperience,
      personalInfo: validPersonalInfo,
      projects: validProjects,
      skills: validSkills,
      summary:
        'Experienced software engineer with a passion for building great products',
    }

    describe('valid data', () => {
      it('should parse valid resume data', () => {
        const result: ResumeData = parseResumeData(validResumeData)
        expect(result).toEqual(validResumeData)
      })

      it('should parse resume data with optional linkedin', () => {
        const { linkedin, ...personalInfoWithoutLinkedIn } = validPersonalInfo
        const dataWithoutLinkedIn: ResumeData = {
          ...validResumeData,
          personalInfo: personalInfoWithoutLinkedIn,
        }
        const result: ResumeData = parseResumeData(dataWithoutLinkedIn)
        expect(result.personalInfo?.linkedin).toBeUndefined()
      })

      it('should parse resume data with optional project url', () => {
        const project = validProjects[0]
        if (!project) throw new Error('Project not found')
        const { url, ...projectWithoutUrl } = project
        const dataWithoutUrl: ResumeData = {
          ...validResumeData,
          projects: [projectWithoutUrl],
        }
        const result: ResumeData = parseResumeData(dataWithoutUrl)
        expect(result.projects[0]?.url).toBeUndefined()
      })

      it('should handle empty arrays', () => {
        const dataWithEmptyArrays: ResumeData = {
          ...validResumeData,
          education: [],
          experience: [],
          projects: [],
        }
        const result: ResumeData = parseResumeData(dataWithEmptyArrays)
        expect(result.experience).toHaveLength(0)
        expect(result.projects).toHaveLength(0)
        expect(result.education).toHaveLength(0)
      })
    })

    describe('invalid data - null/undefined', () => {
      it('should throw on null', () => {
        expect(() => parseResumeData(null)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw on undefined', () => {
        expect(() => parseResumeData(undefined)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw on non-object', () => {
        expect(() => parseResumeData('string')).toThrow(
          'Invalid resume data structure'
        )
        expect(() => parseResumeData(123)).toThrow(
          'Invalid resume data structure'
        )
        expect(() => parseResumeData(true)).toThrow(
          'Invalid resume data structure'
        )
      })
    })

    describe('invalid data - missing fields', () => {
      it('should throw when personalInfo is missing', () => {
        const invalid = { ...validResumeData, personalInfo: undefined }
        expect(() => parseResumeData(invalid)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when summary is missing', () => {
        const invalid = { ...validResumeData, summary: undefined }
        expect(() => parseResumeData(invalid)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when skills is missing', () => {
        const invalid = { ...validResumeData, skills: undefined }
        expect(() => parseResumeData(invalid)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when experience is missing', () => {
        const invalid = { ...validResumeData, experience: undefined }
        expect(() => parseResumeData(invalid)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when projects is missing', () => {
        const invalid = { ...validResumeData, projects: undefined }
        expect(() => parseResumeData(invalid)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when education is missing', () => {
        const invalid = { ...validResumeData, education: undefined }
        expect(() => parseResumeData(invalid)).toThrow(
          'Invalid resume data structure'
        )
      })
    })

    describe('invalid data - wrong types', () => {
      it('should throw when personalInfo fields are wrong type', () => {
        const invalidName = {
          ...validResumeData,
          personalInfo: { ...validPersonalInfo, name: 123 },
        }
        expect(() => parseResumeData(invalidName)).toThrow(
          'Invalid resume data structure'
        )

        const invalidEmail = {
          ...validResumeData,
          personalInfo: { ...validPersonalInfo, email: null },
        }
        expect(() => parseResumeData(invalidEmail)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when summary is not a string', () => {
        const invalid = { ...validResumeData, summary: 123 }
        expect(() => parseResumeData(invalid)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when skills arrays contain non-strings', () => {
        const invalidSkills = {
          ...validResumeData,
          skills: {
            expertise: ['JavaScript', 123, 'React'],
            learning: ['Rust'],
            tools: ['Git'],
          },
        }
        expect(() => parseResumeData(invalidSkills)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when experience is not an array', () => {
        const invalid = { ...validResumeData, experience: 'not an array' }
        expect(() => parseResumeData(invalid)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when experience item is missing required fields', () => {
        const invalidExperience = {
          ...validResumeData,
          experience: [{ company: 'Test', title: 'Developer' }],
        }
        expect(() => parseResumeData(invalidExperience)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when project is missing required fields', () => {
        const invalidProjects = {
          ...validResumeData,
          projects: [{ name: 'Project' }],
        }
        expect(() => parseResumeData(invalidProjects)).toThrow(
          'Invalid resume data structure'
        )
      })

      it('should throw when education is missing required fields', () => {
        const invalidEducation = {
          ...validResumeData,
          education: [{ degree: 'BS' }],
        }
        expect(() => parseResumeData(invalidEducation)).toThrow(
          'Invalid resume data structure'
        )
      })
    })

    describe('edge cases', () => {
      it('should handle multiple experiences', () => {
        const multipleExperience: ResumeData = {
          ...validResumeData,
          experience: [
            ...validExperience,
            {
              achievements: ['Achievement 1', 'Achievement 2'],
              company: 'Another Corp',
              endDate: '2020',
              location: 'New York, NY',
              startDate: '2018',
              title: 'Junior Developer',
            },
          ],
        }
        const result: ResumeData = parseResumeData(multipleExperience)
        expect(result.experience).toHaveLength(2)
      })

      it('should handle special characters in strings', () => {
        const specialChars: ResumeData = {
          ...validResumeData,
          personalInfo: {
            ...validPersonalInfo,
            name: "O'Brien-Smith",
          },
          summary: 'Summary with "quotes" and <tags> and & symbols',
        }
        const result: ResumeData = parseResumeData(specialChars)
        expect(result.personalInfo?.name).toBe("O'Brien-Smith")
        expect(result.summary).toContain('"quotes"')
      })

      it('should handle empty strings', () => {
        const emptyStrings: ResumeData = {
          ...validResumeData,
          summary: '',
        }
        const result: ResumeData = parseResumeData(emptyStrings)
        expect(result.summary).toBe('')
      })

      it('should handle long arrays', () => {
        const longSkills: ResumeData = {
          ...validResumeData,
          skills: {
            expertise: Array.from(
              { length: 50 },
              (_, index: number): string => `Skill ${index + 1}`
            ),
            learning: Array.from(
              { length: 20 },
              (_, index: number): string => `Learning ${index + 1}`
            ),
            tools: Array.from(
              { length: 30 },
              (_, index: number): string => `Tool ${index + 1}`
            ),
          },
        }
        const result: ResumeData = parseResumeData(longSkills)
        expect(result.skills.expertise).toHaveLength(50)
        expect(result.skills.learning).toHaveLength(20)
        expect(result.skills.tools).toHaveLength(30)
      })
    })
  })
})
