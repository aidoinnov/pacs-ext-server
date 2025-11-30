import { TestSection } from './types';

export const getDicomSections = (): TestSection[] => [
  {
    title: '🔍 DICOM 전체 조회 + 할당 여부 확인',
    description: 'project_id 옵셔널화, READ_ALL 권한, check_assignment_for_project 테스트',
    isSequential: false,
    tests: [
      {
        name: 'DICOM Studies 전체 조회 (project_id 없음)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: 'DICOM Studies 프로젝트별 조회 (project_id 있음)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: 'DICOM Series 전체 조회 (project_id 없음)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: 'DICOM Instances 전체 조회 (project_id 없음)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: '할당 여부 확인 (check_assignment_for_project)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: '전체 조회 + 할당 여부 확인 (통합)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: '프로젝트별 조회 + 할당 여부 확인',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: '다른 프로젝트 할당 여부 확인',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: '잘못된 project_id (0) 에러 처리',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: '잘못된 project_id (음수) 에러 처리',
        status: 'pending',
        indentLevel: 0,
      },
    ],
  },
  {
    title: '👤 DICOM Patient API (QIDO-RS 프록시)',
    description: 'Patient 레벨 QIDO-RS 프록시 + RBAC 필터링 + 페이지네이션 테스트',
    isSequential: false,
    tests: [
      {
        name: 'Patient 전체 조회 (project_id 있음)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: 'Patient 페이지네이션 (limit=1)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: 'Patient 필터링 (PatientName)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: 'Patient 필터링 (PatientID)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: 'Patient DICOM JSON 구조 검증',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: 'project_id 없이 조회 (400 에러)',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: '잘못된 project_id (0) 에러 처리',
        status: 'pending',
        indentLevel: 0,
      },
      {
        name: '잘못된 project_id (음수) 에러 처리',
        status: 'pending',
        indentLevel: 0,
      },
    ],
  },
];
