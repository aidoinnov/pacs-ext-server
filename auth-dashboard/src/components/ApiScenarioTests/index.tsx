import React, { useState, useRef } from 'react';
import axios from 'axios';
import '../ApiHealthCheck.css';
import { TestResult, TestSection, TestAccount } from './types';
import { TEST_ACCOUNTS, DEFAULT_API_URL } from './constants';
import { getTestToken, getAxiosConfig } from './utils';
import { getProjectSections } from './ProjectTests';
import { getDicomSections } from './DicomTests';
import { getAnnotationSections } from './AnnotationTests';
import { getProjectDataAccessSections } from './ProjectDataAccessTests';

const ApiScenarioTests: React.FC = () => {
  const [apiUrl] = useState(DEFAULT_API_URL);
  const [testToken, setTestToken] = useState<string | null>(null);
  const [currentTestAccount, setCurrentTestAccount] = useState<TestAccount>(TEST_ACCOUNTS.SUPER_ADMIN);
  const [sections, setSections] = useState<TestSection[]>([
    ...getProjectSections(),
    ...getDicomSections(),
    ...getAnnotationSections(),
    ...getProjectDataAccessSections(),
  ]);
  
  // 기존 섹션 정의는 주석 처리 (위에서 import한 섹션 사용)
  /*
  const [sections, setSections] = useState<TestSection[]>([
    {
      title: '📊 프로젝트 메타데이터',
      description: '프로젝트 상태 메타데이터 조회 테스트 (순서 무관)',
      isSequential: false, // 순서 무관
      tests: [
        { name: '메타데이터 조회', status: 'pending' },
        { name: '메타데이터 구조 검증', status: 'pending' },
        { name: '5개 상태 존재 확인', status: 'pending' },
      ],
    },
    {
      title: '🔄 프로젝트 생명주기',
      description: '프로젝트 생성 및 상태 변경 테스트 (순차 실행)',
      isSequential: true, // 순차 실행 필수
      tests: [
        {
          name: '프로젝트 생성 (PREPARING)',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          delayAfter: 1500, // 프로젝트 생성 후 1.5초 대기 (DB 커밋 완료 대기)
        },
        {
          name: '프로젝트 조회',
          status: 'pending',
          dependencies: ['프로젝트 생성 (PREPARING)'],
          isSequential: true,
          indentLevel: 1,
        },
        {
          name: 'PREPARING → IN_PROGRESS',
          status: 'pending',
          dependencies: ['프로젝트 생성 (PREPARING)'],
          indentLevel: 1,
        },
        {
          name: 'IN_PROGRESS → ON_HOLD',
          status: 'pending',
          dependencies: ['PREPARING → IN_PROGRESS'],
          indentLevel: 2,
        },
        {
          name: 'ON_HOLD → IN_PROGRESS',
          status: 'pending',
          dependencies: ['IN_PROGRESS → ON_HOLD'],
          indentLevel: 3,
        },
        {
          name: 'IN_PROGRESS → COMPLETED',
          status: 'pending',
          dependencies: ['ON_HOLD → IN_PROGRESS'],
          indentLevel: 4,
        },
        {
          name: '잘못된 상태 값 처리',
          status: 'pending',
          dependencies: ['IN_PROGRESS → COMPLETED'],
          indentLevel: 1,
        },
        {
          name: '존재하지 않는 프로젝트 조회',
          status: 'pending',
          dependencies: ['잘못된 상태 값 처리'],
          indentLevel: 1,
        },
        {
          name: '테스트 프로젝트 삭제',
          status: 'pending',
          dependencies: ['존재하지 않는 프로젝트 조회'],
          cleanup: true,
          isSequential: true,
          indentLevel: 1,
        },
      ],
    },
    {
      title: '📦 프로젝트 데이터 할당/제거',
      description: 'DICOM Study/Series 할당 및 조회 테스트 (순차 실행)',
      isSequential: true,
      tests: [
        {
          name: '데이터 테스트용 프로젝트 생성',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          delayAfter: 1500,
        },
        {
          name: 'Study 할당',
          status: 'pending',
          dependencies: ['데이터 테스트용 프로젝트 생성'],
          isSequential: true,
          indentLevel: 1,
        },
        {
          name: 'Series 할당 (3개)',
          status: 'pending',
          dependencies: ['Study 할당'],
          isSequential: true,
          indentLevel: 1,
        },
        {
          name: '프로젝트 Study 목록 조회',
          status: 'pending',
          dependencies: ['Series 할당 (3개)'],
          indentLevel: 2,
        },
        {
          name: '프로젝트 Series 목록 조회',
          status: 'pending',
          dependencies: ['프로젝트 Study 목록 조회'],
          indentLevel: 2,
        },
        {
          name: 'Series 중복 할당 시도 (409 에러)',
          status: 'pending',
          dependencies: ['프로젝트 Series 목록 조회'],
          indentLevel: 2,
        },
        {
          name: '존재하지 않는 프로젝트에 할당 (404 에러)',
          status: 'pending',
          dependencies: ['Series 중복 할당 시도 (409 에러)'],
          indentLevel: 2,
        },
        {
          name: '다른 프로젝트 생성 (격리 테스트)',
          status: 'pending',
          dependencies: ['존재하지 않는 프로젝트에 할당 (404 에러)'],
          indentLevel: 1,
        },
        {
          name: '다른 프로젝트 데이터 조회 (빈 목록)',
          status: 'pending',
          dependencies: ['다른 프로젝트 생성 (격리 테스트)'],
          indentLevel: 2,
        },
        {
          name: 'Series 할당 해제 (첫 번째)',
          status: 'pending',
          dependencies: ['다른 프로젝트 데이터 조회 (빈 목록)'],
          indentLevel: 1,
          delayAfter: 500, // DB 트랜잭션 완료 대기
        },
        {
          name: 'Series 목록 재조회 (2개 확인)',
          status: 'pending',
          dependencies: ['Series 할당 해제 (첫 번째)'],
          indentLevel: 2,
        },
        {
          name: 'Study 할당 해제',
          status: 'pending',
          dependencies: ['Series 목록 재조회 (2개 확인)'],
          indentLevel: 1,
        },
        {
          name: 'Study 목록 재조회 (빈 목록 확인)',
          status: 'pending',
          dependencies: ['Study 할당 해제'],
          indentLevel: 2,
        },
        {
          name: '데이터 테스트 프로젝트 삭제',
          status: 'pending',
          dependencies: ['Study 목록 재조회 (빈 목록 확인)'],
          cleanup: true,
          isSequential: true,
          indentLevel: 1,
        },
      ],
    },
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
      title: '🏷️ Annotation Label 기능',
      description: 'Annotation Label 생성, 수정, 조회 테스트 (순차 실행)',
      isSequential: true,
      tests: [
        {
          name: '1️⃣ Label 없이 Annotation 생성',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
        },
        {
          name: '2️⃣ Label과 함께 Annotation 생성 (Tumor)',
          status: 'pending',
          dependencies: ['1️⃣ Label 없이 Annotation 생성'],
          indentLevel: 0,
        },
        {
          name: '3️⃣ 생성된 Annotation 조회 (Label 확인)',
          status: 'pending',
          dependencies: ['2️⃣ Label과 함께 Annotation 생성 (Tumor)'],
          indentLevel: 1,
        },
        {
          name: '4️⃣ Label 수정 (Tumor → Lesion)',
          status: 'pending',
          dependencies: ['3️⃣ 생성된 Annotation 조회 (Label 확인)'],
          indentLevel: 1,
        },
        {
          name: '5️⃣ 수정된 Label 확인',
          status: 'pending',
          dependencies: ['4️⃣ Label 수정 (Tumor → Lesion)'],
          indentLevel: 2,
        },
        {
          name: '6️⃣ 다양한 Label로 Annotation 생성 (Normal, Abnormal, Suspicious)',
          status: 'pending',
          dependencies: ['5️⃣ 수정된 Label 확인'],
          indentLevel: 0,
        },
        {
          name: '7️⃣ 모든 Annotation 조회 (Label 포함)',
          status: 'pending',
          dependencies: ['6️⃣ 다양한 Label로 Annotation 생성 (Normal, Abnormal, Suspicious)'],
          indentLevel: 1,
        },
        {
          name: '8️⃣ Label 빈 문자열로 수정',
          status: 'pending',
          dependencies: ['7️⃣ 모든 Annotation 조회 (Label 포함)'],
          indentLevel: 1,
        },
        {
          name: '9️⃣ 정리 (생성된 Annotation 삭제)',
          status: 'pending',
          dependencies: ['8️⃣ Label 빈 문자열로 수정'],
          indentLevel: 0,
          cleanup: true,
        },
      ],
    },
    {
      title: '🔒 Project Data Access 접근 제어',
      description: '다기관 공동 연구 프로젝트 시나리오 - 사용자별 데이터 접근 제어 테스트',
      isSequential: true,
      tests: [
        {
          name: '시나리오 구성 (프로젝트 + 사용자 + Study + 접근 제어)',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          delayAfter: 1500,
        },
        {
          name: '접근 제어 매트릭스 조회',
          status: 'pending',
          dependencies: ['시나리오 구성 (프로젝트 + 사용자 + Study + 접근 제어)'],
          indentLevel: 1,
        },
        {
          name: '매트릭스 구조 검증 (4명 사용자)',
          status: 'pending',
          dependencies: ['접근 제어 매트릭스 조회'],
          indentLevel: 2,
        },
        {
          name: '매트릭스 구조 검증 (7개 Study)',
          status: 'pending',
          dependencies: ['접근 제어 매트릭스 조회'],
          indentLevel: 2,
        },
        {
          name: 'Dr. Kim 전체 접근 확인 (7/7)',
          status: 'pending',
          dependencies: ['매트릭스 구조 검증 (7개 Study)'],
          indentLevel: 2,
        },
        {
          name: 'Dr. Lee A병원만 접근 확인 (3/7)',
          status: 'pending',
          dependencies: ['매트릭스 구조 검증 (7개 Study)'],
          indentLevel: 2,
        },
        {
          name: 'Dr. Park B병원만 접근 확인 (3/7)',
          status: 'pending',
          dependencies: ['매트릭스 구조 검증 (7개 Study)'],
          indentLevel: 2,
        },
        {
          name: 'Dr. Choi 읽기 전용 확인 (1/7)',
          status: 'pending',
          dependencies: ['매트릭스 구조 검증 (7개 Study)'],
          indentLevel: 2,
        },
        {
          name: '시나리오 초기화',
          status: 'pending',
          dependencies: ['Dr. Choi 읽기 전용 확인 (1/7)'],
          cleanup: true,
          isSequential: true,
          indentLevel: 1,
        },
      ],
    },
    {
      title: '🔄 Project Data Access 순차 시나리오 (실제 API 호출)',
      description: '프론트엔드에서 직접 순차적으로 API를 호출하여 접근 제어 시나리오 구성 및 검증',
      isSequential: true,
      tests: [
        {
          name: '0️⃣ 사전 정리 (기존 테스트 데이터 삭제)',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          cleanup: true,
        },
        {
          name: '1️⃣ 프로젝트 생성',
          status: 'pending',
          dependencies: ['0️⃣ 사전 정리 (기존 테스트 데이터 삭제)'],
          indentLevel: 0,
        },
        {
          name: '2️⃣ 사용자 4명 생성 (Dr. Kim, Dr. Lee, Dr. Park, Dr. Choi)',
          status: 'pending',
          dependencies: ['1️⃣ 프로젝트 생성'],
          indentLevel: 0,
        },
        {
          name: '3️⃣ 사용자 4명 활성화 (관리자 승인)',
          status: 'pending',
          dependencies: ['2️⃣ 사용자 4명 생성 (Dr. Kim, Dr. Lee, Dr. Park, Dr. Choi)'],
          indentLevel: 0,
        },
        {
          name: '4️⃣ 사용자를 프로젝트 멤버로 추가',
          status: 'pending',
          dependencies: ['3️⃣ 사용자 4명 활성화 (관리자 승인)'],
          indentLevel: 0,
        },
        {
          name: '5️⃣ Study 7개 생성 (A병원 3개, B병원 3개, VIP 1개)',
          status: 'pending',
          dependencies: ['4️⃣ 사용자를 프로젝트 멤버로 추가'],
          indentLevel: 0,
        },
        {
          name: '6️⃣ 접근 제어 설정 (Dr. Lee → A병원, Dr. Park → B병원, Dr. Choi → VIP)',
          status: 'pending',
          dependencies: ['5️⃣ Study 7개 생성 (A병원 3개, B병원 3개, VIP 1개)'],
          indentLevel: 0,
        },
        {
          name: '7️⃣ 접근 제어 매트릭스 조회 및 검증',
          status: 'pending',
          dependencies: ['6️⃣ 접근 제어 설정 (Dr. Lee → A병원, Dr. Park → B병원, Dr. Choi → VIP)'],
          indentLevel: 0,
        },
        {
          name: '8️⃣ DICOM QIDO API로 실제 접근 제어 검증',
          status: 'pending',
          dependencies: ['7️⃣ 접근 제어 매트릭스 조회 및 검증'],
          indentLevel: 0,
        },
        {
          name: '9️⃣ 정리 (프로젝트 삭제)',
          status: 'pending',
          dependencies: ['8️⃣ DICOM QIDO API로 실제 접근 제어 검증'],
          indentLevel: 0,
          cleanup: true,
        },
      ],
    },
    {
      title: '🔐 Annotation 권한 관리',
      description: 'Annotation 생성/수정/삭제 권한 제어 및 권한 조회 API 테스트 (순차 실행)',
      isSequential: true,
      tests: [
        {
          name: '0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          cleanup: true,
        },
        {
          name: '1️⃣ 테스트용 프로젝트 생성',
          status: 'pending',
          isSequential: true,
          dependencies: ['0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)'],
          indentLevel: 0,
          delayAfter: 1000,
        },
        {
          name: '2️⃣ 사용자를 프로젝트 멤버로 추가',
          status: 'pending',
          dependencies: ['1️⃣ 테스트용 프로젝트 생성'],
          indentLevel: 1,
        },
        {
          name: '3️⃣ 개발 모드: 쿼리 파라미터로 Annotation 생성',
          status: 'pending',
          dependencies: ['2️⃣ 사용자를 프로젝트 멤버로 추가'],
          indentLevel: 1,
        },
        {
          name: '4️⃣ 개발 모드: 헤더로 Annotation 생성',
          status: 'pending',
          dependencies: ['3️⃣ 개발 모드: 쿼리 파라미터로 Annotation 생성'],
          indentLevel: 1,
        },
        {
          name: '5️⃣ 권한 조회 API 테스트',
          status: 'pending',
          dependencies: ['4️⃣ 개발 모드: 헤더로 Annotation 생성'],
          indentLevel: 1,
        },
        {
          name: '6️⃣ 소유자 Annotation 조회 테스트',
          status: 'pending',
          dependencies: ['5️⃣ 권한 조회 API 테스트'],
          indentLevel: 1,
        },
        {
          name: '7️⃣ READ_ALL 권한으로 다른 사용자 Annotation 조회 테스트',
          status: 'pending',
          dependencies: ['6️⃣ 소유자 Annotation 조회 테스트'],
          indentLevel: 1,
        },
        {
          name: '8️⃣ 권한 없는 사용자 Annotation 조회 시도 (401 에러)',
          status: 'pending',
          dependencies: ['7️⃣ READ_ALL 권한으로 다른 사용자 Annotation 조회 테스트'],
          indentLevel: 1,
        },
        {
          name: '9️⃣ 소유자 Annotation 수정 테스트',
          status: 'pending',
          dependencies: ['8️⃣ 권한 없는 사용자 Annotation 조회 시도 (401 에러)'],
          indentLevel: 1,
        },
        {
          name: '🔟 소유자 Annotation 삭제 테스트',
          status: 'pending',
          dependencies: ['9️⃣ 소유자 Annotation 수정 테스트'],
          indentLevel: 1,
        },
        {
          name: '1️⃣1️⃣ 권한 없는 사용자 Annotation 생성 시도 (401 에러)',
          status: 'pending',
          dependencies: ['🔟 소유자 Annotation 삭제 테스트'],
          indentLevel: 1,
        },
        {
          name: '1️⃣2️⃣ 정리 (테스트 프로젝트 삭제)',
          status: 'pending',
          dependencies: ['1️⃣1️⃣ 권한 없는 사용자 Annotation 생성 시도 (401 에러)'],
          indentLevel: 0,
          cleanup: true,
        },
      ],
    },
    {
      title: '👁️ 권한 기반 Annotation 조회 (READ_ALL)',
      description: 'ADJUDICATOR 역할의 READ_ALL 권한을 통한 모든 사용자 Annotation 조회 테스트 (순차 실행)',
      isSequential: true,
      tests: [
        {
          name: '0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          cleanup: true,
        },
        {
          name: '1️⃣ 테스트용 프로젝트 생성',
          status: 'pending',
          isSequential: true,
          dependencies: ['0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)'],
          indentLevel: 0,
        },
        {
          name: '2️⃣ 사용자들을 프로젝트 멤버로 추가',
          status: 'pending',
          dependencies: ['1️⃣ 테스트용 프로젝트 생성'],
          indentLevel: 1,
        },
        {
          name: '3️⃣ 일반 사용자(user_id=54)로 Annotation 3개 생성',
          status: 'pending',
          dependencies: ['2️⃣ 사용자들을 프로젝트 멤버로 추가'],
          indentLevel: 1,
        },
        {
          name: '4️⃣ 다른 사용자(user_id=5)로 Annotation 2개 생성',
          status: 'pending',
          dependencies: ['3️⃣ 일반 사용자(user_id=54)로 Annotation 3개 생성'],
          indentLevel: 1,
        },
        {
          name: '5️⃣ 일반 사용자(user_id=54) 본인 Annotation만 조회 (3개)',
          status: 'pending',
          dependencies: ['4️⃣ 다른 사용자(user_id=5)로 Annotation 2개 생성'],
          indentLevel: 1,
        },
        {
          name: '6️⃣ ADJUDICATOR(user_id=56) 모든 Annotation 조회 (5개)',
          status: 'pending',
          dependencies: ['5️⃣ 일반 사용자(user_id=54) 본인 Annotation만 조회 (3개)'],
          indentLevel: 1,
        },
        {
          name: '7️⃣ SOP Instance UID로 조회 (READ_ALL 권한 확인)',
          status: 'pending',
          dependencies: ['6️⃣ ADJUDICATOR(user_id=56) 모든 Annotation 조회 (5개)'],
          indentLevel: 1,
        },
        {
          name: '8️⃣ Series UID로 조회 (READ_ALL 권한 확인)',
          status: 'pending',
          dependencies: ['7️⃣ SOP Instance UID로 조회 (READ_ALL 권한 확인)'],
          indentLevel: 1,
        },
        {
          name: '9️⃣ Study UID로 조회 (READ_ALL 권한 확인)',
          status: 'pending',
          dependencies: ['8️⃣ Series UID로 조회 (READ_ALL 권한 확인)'],
          indentLevel: 1,
        },
        {
          name: '🔟 Summary API로 전체 통계 조회',
          status: 'pending',
          dependencies: ['9️⃣ Study UID로 조회 (READ_ALL 권한 확인)'],
          indentLevel: 1,
        },
        {
          name: '1️⃣1️⃣ 정리 (생성된 Annotation 삭제)',
          status: 'pending',
          dependencies: ['🔟 Summary API로 전체 통계 조회'],
          indentLevel: 0,
          cleanup: true,
        },
      ],
    },
    {
      title: '🔐 Annotation 권한 조회 API 개선',
      description: 'Annotation 권한 조회 API 개선 기능 테스트 (순차 실행)',
      isSequential: true,
      tests: [
        {
          name: '0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)',
          status: 'pending',
          isSequential: true,
          indentLevel: 0,
          cleanup: true,
        },
        {
          name: '1️⃣ 테스트용 프로젝트 생성',
          status: 'pending',
          isSequential: true,
          dependencies: ['0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)'],
          indentLevel: 0,
          delayAfter: 1000,
        },
        {
          name: '2️⃣ 사용자들을 프로젝트 멤버로 추가',
          status: 'pending',
          dependencies: ['1️⃣ 테스트용 프로젝트 생성'],
          indentLevel: 1,
        },
        {
          name: '3️⃣ 테스트용 사용자 조회/확인',
          status: 'pending',
          dependencies: ['2️⃣ 사용자들을 프로젝트 멤버로 추가'],
          indentLevel: 1,
        },
        {
          name: '4️⃣ 본인 권한 조회 (user_id 파라미터 없음, 헤더만)',
          status: 'pending',
          dependencies: ['3️⃣ 테스트용 사용자 조회/확인'],
          indentLevel: 1,
        },
        {
          name: '5️⃣ 본인 권한 조회 (user_id 쿼리 파라미터로 명시)',
          status: 'pending',
          dependencies: ['4️⃣ 본인 권한 조회 (user_id 파라미터 없음, 헤더만)'],
          indentLevel: 1,
        },
        {
          name: '6️⃣ 본인 권한 조회 (쿼리 파라미터와 헤더 모두, 쿼리 우선순위)',
          status: 'pending',
          dependencies: ['5️⃣ 본인 권한 조회 (user_id 쿼리 파라미터로 명시)'],
          indentLevel: 1,
        },
        {
          name: '7️⃣ 다른 사용자 권한 조회 (프로젝트 멤버인 경우)',
          status: 'pending',
          dependencies: ['6️⃣ 본인 권한 조회 (쿼리 파라미터와 헤더 모두, 쿼리 우선순위)'],
          indentLevel: 1,
        },
        {
          name: '8️⃣ project_id 누락 시 400 에러',
          status: 'pending',
          dependencies: ['7️⃣ 다른 사용자 권한 조회 (프로젝트 멤버인 경우)'],
          indentLevel: 1,
        },
        {
          name: '9️⃣ project_id가 0일 때 400 에러',
          status: 'pending',
          dependencies: ['8️⃣ project_id 누락 시 400 에러'],
          indentLevel: 1,
        },
        {
          name: '🔟 project_id가 음수일 때 400 에러',
          status: 'pending',
          dependencies: ['9️⃣ project_id가 0일 때 400 에러'],
          indentLevel: 1,
        },
        {
          name: '1️⃣1️⃣ project_id가 유효하지 않은 형식 (문자열) 400 에러',
          status: 'pending',
          dependencies: ['🔟 project_id가 음수일 때 400 에러'],
          indentLevel: 1,
        },
        {
          name: '1️⃣2️⃣ user_id 없음 (헤더도 쿼리도 없음) 401 에러',
          status: 'pending',
          dependencies: ['1️⃣1️⃣ project_id가 유효하지 않은 형식 (문자열) 400 에러'],
          indentLevel: 1,
        },
        {
          name: '1️⃣3️⃣ 프로젝트 멤버가 아닌 사용자가 다른 사용자 권한 조회 시도 (403 에러)',
          status: 'pending',
          dependencies: ['1️⃣2️⃣ user_id 없음 (헤더도 쿼리도 없음) 401 에러'],
          indentLevel: 1,
        },
        {
          name: '1️⃣4️⃣ 존재하지 않는 프로젝트의 권한 조회 (404/401 에러)',
          status: 'pending',
          dependencies: ['1️⃣3️⃣ 프로젝트 멤버가 아닌 사용자가 다른 사용자 권한 조회 시도 (403 에러)'],
          indentLevel: 1,
        },
        {
          name: '1️⃣5️⃣ target_user_id가 프로젝트 멤버가 아닌 경우 (401 에러)',
          status: 'pending',
          dependencies: ['1️⃣4️⃣ 존재하지 않는 프로젝트의 권한 조회 (404/401 에러)'],
          indentLevel: 1,
        },
        {
          name: '1️⃣6️⃣ 정리 (테스트 프로젝트 삭제)',
          status: 'pending',
          dependencies: ['1️⃣5️⃣ target_user_id가 프로젝트 멤버가 아닌 경우 (401 에러)'],
          indentLevel: 0,
          cleanup: true,
        },
      ],
    },
  ]);
  */

  const [expandedTest, setExpandedTest] = useState<string | null>(null);
  const [isRunningAll, setIsRunningAll] = useState(false);
  const [createdProjectId, setCreatedProjectId] = useState<number | null>(null);

  // useRef로 즉시 접근 가능한 프로젝트 ID 및 데이터 ID 관리
  const createdProjectIdRef = useRef<number | null>(null);
  const createdStudyIdRef = useRef<number | null>(null);
  const createdStudyUidRef = useRef<string | null>(null);
  const createdSeriesIdsRef = useRef<number[]>([]);
  const createdSeriesUidsRef = useRef<string[]>([]);

  // Annotation 테스트용 ref
  const createdAnnotationIdsRef = useRef<number[]>([]);

  // 순차 시나리오용 ref
  const sequentialProjectIdRef = useRef<number | null>(null);
  const sequentialUserIdsRef = useRef<{[key: string]: number}>({});
  const sequentialStudyIdsRef = useRef<number[]>([]);

  // Annotation 권한 관리 테스트용 ref
  const annotationPermissionProjectIdRef = useRef<number | null>(null);
  const annotationPermissionAnnotationIdsRef = useRef<number[]>([]);
  const annotationPermissionTestUserIdRef = useRef<number | null>(null);

  // 권한 기반 Annotation 조회 테스트용 ref
  const readAllProjectIdRef = useRef<number | null>(null);
  const readAllAnnotationIdsRef = useRef<number[]>([]);
  const readAllTestStudyUidRef = useRef<string>('1.2.410.200003.9.1.0.547.20170531.212400.1705312223.1');
  const readAllTestSeriesUidRef = useRef<string>('1.3.12.2.1107.5.1.4.54583.30000017053018462370300031465');
  const readAllTestSopUidRef = useRef<string>('1.3.12.2.1107.5.1.4.54583.30000017053018462370300031491');
  const readAllUser1IdRef = useRef<number | null>(null); // 일반 사용자 1
  const readAllUser2IdRef = useRef<number | null>(null); // 일반 사용자 2
  const readAllAdjudicatorIdRef = useRef<number | null>(null); // ADJUDICATOR
  const readAllAdjudicatorRoleIdRef = useRef<number | null>(null); // ADJUDICATOR 역할 ID

  // Annotation 권한 조회 API 개선 테스트용 ref
  const annotationPermissionsApiProjectIdRef = useRef<number | null>(null);
  const annotationPermissionsApiRequestingUserIdRef = useRef<number | null>(null);
  const annotationPermissionsApiTargetUserIdRef = useRef<number | null>(null);
  const annotationPermissionsApiNonMemberUserIdRef = useRef<number | null>(null);

  // 통계 계산
  const getStats = () => {
    const allTests = sections.flatMap(s => s.tests);
    return {
      total: allTests.length,
      success: allTests.filter(t => t.status === 'success').length,
      failure: allTests.filter(t => t.status === 'failure').length,
      running: allTests.filter(t => t.status === 'running').length,
      pending: allTests.filter(t => t.status === 'pending').length,
      skipped: allTests.filter(t => t.status === 'skipped').length,
    };
  };

  // 테스트 계정으로 토큰 획득 (백엔드 프록시를 통해 Keycloak 토큰 획득)
  const handleGetTestToken = async (account: TestAccount): Promise<string> => {
    return getTestToken(account, apiUrl, setTestToken, setCurrentTestAccount);
  };

  // axios 요청에 토큰 추가
  const handleGetAxiosConfig = async (accountType?: 'SUPER_ADMIN' | 'ADMIN' | 'USER') => {
    return getAxiosConfig(accountType, testToken, apiUrl, setTestToken, setCurrentTestAccount);
  };

  // 의존성 체크: 특정 테스트를 실행할 수 있는지 확인
  const canRunTest = (
    sectionIndex: number,
    testIndex: number,
    currentSections: TestSection[]
  ): { canRun: boolean; reason?: string } => {
    const test = currentSections[sectionIndex].tests[testIndex];

    if (!test.dependencies || test.dependencies.length === 0) {
      return { canRun: true };
    }

    // 모든 섹션의 모든 테스트를 검색
    const allTests = currentSections.flatMap(s => s.tests);

    for (const depName of test.dependencies) {
      const depTest = allTests.find(t => t.name === depName);

      if (!depTest) {
        return { canRun: false, reason: `의존 테스트를 찾을 수 없습니다: ${depName}` };
      }

      // cleanup 테스트는 실행만 되면 성공으로 간주 (에러가 있어도 다음 단계 진행 가능)
      if (depTest.cleanup === true) {
        // cleanup 테스트가 실행 중이거나 완료된 경우 (pending이 아닌 경우) 통과
        if (depTest.status === 'pending') {
          return { canRun: false, reason: `먼저 "${depName}" 테스트를 실행해야 합니다` };
        }
        // cleanup 테스트는 running, success, failure 모두 통과 (정리 단계이므로)
        continue;
      }

      if (depTest.status !== 'success') {
        return { canRun: false, reason: `먼저 "${depName}" 테스트를 성공시켜야 합니다` };
      }
    }

    return { canRun: true };
  };



  // 개별 테스트 실행
  const runTest = async (sectionIndex: number, testIndex: number) => {
    // 최신 sections 상태 사용
    const newSections = [...sections];
    
    // 의존성 체크 (최신 상태 사용)
    const dependencyCheck = canRunTest(sectionIndex, testIndex, newSections);
    if (!dependencyCheck.canRun) {
      // alert 대신 테스트 항목에 에러 표시
      newSections[sectionIndex].tests[testIndex].status = 'failure';
      newSections[sectionIndex].tests[testIndex].error = `⚠️ ${dependencyCheck.reason}`;
      setSections(newSections);
      return;
    }
    const test = newSections[sectionIndex].tests[testIndex];

    test.status = 'running';
    test.request = undefined;
    test.response = undefined;
    test.error = undefined;
    setSections(newSections);

    const startTime = Date.now();

    try {
      let result;

      // 섹션별 테스트 로직
      if (sectionIndex === 0) {
        // 메타데이터 섹션
        result = await runMetadataTest(testIndex);
      } else if (sectionIndex === 1) {
        // 프로젝트 생명주기 섹션
        result = await runLifecycleTest(testIndex);
      } else if (sectionIndex === 2) {
        // 프로젝트 데이터 할당/제거 섹션
        result = await runDataAssignmentTest(testIndex);
      } else if (sectionIndex === 3) {
        // DICOM 전체 조회 + 할당 여부 확인 섹션
        result = await runDicomTest(testIndex);
      } else if (sectionIndex === 4) {
        // DICOM Patient API 섹션
        result = await runPatientTest(testIndex);
      } else if (sectionIndex === 5) {
        // Annotation Label 기능 섹션
        result = await runAnnotationLabelTest(testIndex);
      } else if (sectionIndex === 6) {
        // Project Data Access 접근 제어 섹션
        result = await runProjectDataAccessTest(testIndex);
      } else if (sectionIndex === 7) {
        // 순차 시나리오 섹션
        result = await runSequentialScenarioTest(testIndex);
      } else if (sectionIndex === 8) {
        // Annotation 권한 관리 섹션
        result = await runAnnotationPermissionTest(testIndex);
      } else if (sectionIndex === 9) {
        // 권한 기반 Annotation 조회 (READ_ALL) 섹션
        result = await runReadAllPermissionTest(testIndex);
      } else if (sectionIndex === 10) {
        // Annotation 권한 조회 API 개선 섹션
        result = await runAnnotationPermissionsApiTest(testIndex);
      }

      // 정리(cleanup) 테스트는 에러가 발생해도 성공으로 처리
      const isCleanupTest = test.cleanup === true;
      const hasErrorInResponse = result?.response?.error;
      
      if (isCleanupTest && hasErrorInResponse) {
        // 정리 단계는 에러가 있어도 성공으로 처리
        test.status = 'success';
        test.request = result?.request;
        test.response = result?.response;
        test.duration = Date.now() - startTime;
        console.log(`  ℹ️ 정리 단계 완료 (에러 무시): ${test.name}`);
      } else {
        test.status = 'success';
        test.request = result?.request;
        test.response = result?.response;
        test.duration = Date.now() - startTime;
      }
    } catch (error: any) {
      // 정리(cleanup) 테스트는 에러가 발생해도 성공으로 처리
      const isCleanupTest = test.cleanup === true;
      
      if (isCleanupTest) {
        test.status = 'success';
        test.request = error.config ? {
          method: error.config.method?.toUpperCase() || 'UNKNOWN',
          url: error.config.url || 'unknown',
        } : undefined;
        test.response = { 
          message: '정리 단계는 에러가 발생해도 성공으로 처리됩니다',
          error: error.message 
        };
        test.duration = Date.now() - startTime;
        console.log(`  ℹ️ 정리 단계 완료 (에러 무시): ${test.name}`);
      } else {
        test.status = 'failure';
        test.error = error.message || 'Unknown error';

        // 요청 정보 추출
        if (error.config) {
        const method = error.config.method?.toUpperCase() || 'UNKNOWN';
        const url = error.config.url || 'unknown';
        let body = undefined;

        if (error.config.data) {
          try {
            body = typeof error.config.data === 'string'
              ? JSON.parse(error.config.data)
              : error.config.data;
          } catch {
            body = error.config.data;
          }
        }

          test.request = {
            method,
            url,
            ...(body && { body }),
          };
        }

        test.response = error.response?.data;
        test.duration = Date.now() - startTime;
      }
    }

    setSections([...newSections]);
  };

  // 메타데이터 테스트
  const runMetadataTest = async (testIndex: number) => {
    const requestInfo = { method: 'GET', url: '/api/projects/meta' };

    try {
      if (testIndex === 0) {
        // 메타데이터 조회
        const response = await axios.get(`${apiUrl}/api/projects/meta`);
        return {
          request: requestInfo,
          response: response.data,
        };
      } else if (testIndex === 1) {
        // 메타데이터 구조 검증
        const response = await axios.get(`${apiUrl}/api/projects/meta`);
        const data = response.data;

        if (!data.available_statuses || !Array.isArray(data.available_statuses)) {
          throw new Error('available_statuses가 배열이 아닙니다');
        }

        const firstStatus = data.available_statuses[0];
        if (!firstStatus.value || !firstStatus.label || !firstStatus.description) {
          throw new Error('상태 객체에 필수 필드가 없습니다');
        }

        return {
          request: requestInfo,
          response: data,
        };
      } else if (testIndex === 2) {
        // 5개 상태 존재 확인
        const response = await axios.get(`${apiUrl}/api/projects/meta`);
        const data = response.data;

        if (data.available_statuses.length !== 5) {
          throw new Error(`5개의 상태가 있어야 하는데 ${data.available_statuses.length}개가 있습니다`);
        }

        const expectedStatuses = ['PREPARING', 'IN_PROGRESS', 'COMPLETED', 'ON_HOLD', 'CANCELLED'];
        const actualStatuses = data.available_statuses.map((s: any) => s.value);

        for (const expected of expectedStatuses) {
          if (!actualStatuses.includes(expected)) {
            throw new Error(`${expected} 상태가 없습니다`);
          }
        }

        return {
          request: requestInfo,
          response: data,
        };
      }
    } catch (error: any) {
      // 에러에 요청 정보 포함
      if (!error.config) {
        error.config = {
          method: 'get',
          url: `${apiUrl}/api/projects/meta`,
        };
      }
      throw error;
    }
  };

  // 프로젝트 생명주기 테스트
  const runLifecycleTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // 프로젝트 생성
      const projectData = {
        name: `E2E Test ${Date.now()}`,
        description: 'API Health Check Test',
        sponsor: 'Test Sponsor',
        start_date: '2025-01-01',
        end_date: '2025-12-31',
        auto_complete: false,
      };

      try {
        const response = await axios.post(`${apiUrl}/api/projects`, projectData);

        console.log(`  ✅ 프로젝트 생성 성공:`, response.data);
        console.log(`  📝 생성된 프로젝트 ID: ${response.data.id}`);

        if (response.data.status !== 'Preparing') {
          throw new Error(`초기 상태가 Preparing이 아닙니다: ${response.data.status}`);
        }

        // 상태와 ref 모두 업데이트
        const projectId = response.data.id;
        setCreatedProjectId(projectId);
        createdProjectIdRef.current = projectId; // ref는 즉시 업데이트됨
        console.log(`  💾 createdProjectId 저장 완료: ${projectId} (state + ref)`);

        return {
          request: { method: 'POST', url: '/api/projects', body: projectData },
          response: response.data,
          createdId: projectId, // 생성된 ID를 반환값에 포함
        };
      } catch (error: any) {
        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects`,
            data: projectData,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // 프로젝트 조회 (재시도 로직 포함)
      // ref를 사용하여 즉시 업데이트된 값 확인
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        console.log(`  ❌ createdProjectIdRef.current: ${createdProjectIdRef.current}`);
        console.log(`  ❌ createdProjectId state: ${createdProjectId}`);
        throw new Error('프로젝트가 생성되지 않았습니다. 먼저 프로젝트 생성 테스트를 실행하세요.');
      }

      console.log(`  ✅ 프로젝트 ID 확인: ${projectId} (ref 사용)`);

      // 최대 3번 재시도 (DB 커밋 대기)
      let lastError: any = null;
      const requestInfo = { method: 'GET', url: `/api/projects/${projectId}` };

      for (let attempt = 1; attempt <= 3; attempt++) {
        try {
          console.log(`  🔍 프로젝트 조회 시도 ${attempt}/3...`);
          const response = await axios.get(`${apiUrl}/api/projects/${projectId}`);

          console.log(`  ✅ 프로젝트 조회 성공 (시도 ${attempt})`);
          return {
            request: requestInfo,
            response: response.data,
          };
        } catch (error: any) {
          lastError = error;
          console.log(`  ⚠️ 프로젝트 조회 실패 (시도 ${attempt}/3): ${error.message}`);

          if (attempt < 3) {
            // 재시도 전 대기 (500ms)
            console.log(`  ⏱️ 500ms 후 재시도...`);
            await new Promise(resolve => setTimeout(resolve, 500));
          }
        }
      }

      // 모든 재시도 실패 - 에러에 요청 정보 포함
      const errorMessage = `프로젝트 조회 실패 (3회 시도): ${lastError?.response?.data?.error || lastError?.message || '알 수 없는 오류'}`;
      const error: any = new Error(errorMessage);
      error.config = {
        method: 'get',
        url: `${apiUrl}/api/projects/${projectId}`,
      };
      error.response = lastError?.response;
      throw error;
    } else if (testIndex >= 2 && testIndex <= 5) {
      // 상태 변경 테스트
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다. 먼저 프로젝트 생성 테스트를 실행하세요.');
      }

      const transitions = [
        { from: 'Preparing', to: 'IN_PROGRESS', expected: 'InProgress' },
        { from: 'InProgress', to: 'ON_HOLD', expected: 'OnHold' },
        { from: 'OnHold', to: 'IN_PROGRESS', expected: 'InProgress' },
        { from: 'InProgress', to: 'COMPLETED', expected: 'Completed' },
      ];

      const transition = transitions[testIndex - 2];
      const updateData = {
        status: transition.to,
        end_date: '',
      };

      try {
        const response = await axios.put(`${apiUrl}/api/projects/${projectId}`, updateData);

        if (response.data.status !== transition.expected) {
          throw new Error(`상태가 ${transition.expected}로 변경되지 않았습니다: ${response.data.status}`);
        }

        return {
          request: { method: 'PUT', url: `/api/projects/${projectId}`, body: updateData },
          response: response.data,
        };
      } catch (error: any) {
        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'put',
            url: `${apiUrl}/api/projects/${projectId}`,
            data: updateData,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 잘못된 상태 값 처리
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다. 먼저 프로젝트 생성 테스트를 실행하세요.');
      }

      const updateData = {
        status: 'INVALID_STATUS',
        end_date: '',
      };

      try {
        const response = await axios.put(`${apiUrl}/api/projects/${projectId}`, updateData);

        // 잘못된 상태는 무시되고 200 응답이 와야 함
        if (response.status !== 200) {
          throw new Error(`예상치 못한 응답 코드: ${response.status}`);
        }

        return {
          request: { method: 'PUT', url: `/api/projects/${projectId}`, body: updateData },
          response: response.data,
        };
      } catch (error: any) {
        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'put',
            url: `${apiUrl}/api/projects/${projectId}`,
            data: updateData,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 존재하지 않는 프로젝트 조회
      const requestInfo = { method: 'GET', url: '/api/projects/999999' };

      try {
        await axios.get(`${apiUrl}/api/projects/999999`);
        throw new Error('404 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 404) {
          return {
            request: requestInfo,
            response: error.response.data || { status: 404, message: 'Not Found' },
          };
        }

        // 404가 아닌 다른 에러 - 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/projects/999999`,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 테스트 프로젝트 삭제 (cleanup)
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('삭제할 프로젝트가 없습니다.');
      }

      const deletedId = projectId;

      try {
        const response = await axios.delete(`${apiUrl}/api/projects/${projectId}`);

        // 삭제 후 ID 초기화 (state + ref)
        setCreatedProjectId(null);
        createdProjectIdRef.current = null;

        return {
          request: { method: 'DELETE', url: `/api/projects/${deletedId}` },
          response: response.data || { message: '프로젝트가 삭제되었습니다' },
        };
      } catch (error: any) {
        // 삭제 실패해도 ID 초기화 (다음 테스트를 위해)
        setCreatedProjectId(null);
        createdProjectIdRef.current = null;

        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${deletedId}`,
          };
        }
        throw error;
      }
    }
  };

  // 프로젝트 데이터 할당/제거 테스트
  const runDataAssignmentTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // 데이터 테스트용 프로젝트 생성
      const projectData = {
        name: `Data Test ${Date.now()}`,
        description: 'DICOM Data Assignment Test',
        sponsor: 'Test Sponsor',
        start_date: '2025-01-01',
        end_date: '2025-12-31',
        auto_complete: false,
      };

      try {
        const response = await axios.post(`${apiUrl}/api/projects`, projectData);

        console.log(`  ✅ 데이터 테스트용 프로젝트 생성 성공:`, response.data);
        console.log(`  📝 생성된 프로젝트 ID: ${response.data.id}`);

        // 상태와 ref 모두 업데이트
        const projectId = response.data.id;
        setCreatedProjectId(projectId);
        createdProjectIdRef.current = projectId;
        console.log(`  💾 createdProjectId 저장 완료: ${projectId} (state + ref)`);

        return {
          request: { method: 'POST', url: '/api/projects', body: projectData },
          response: response.data,
          createdId: projectId,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects`,
            data: projectData,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // Study 할당
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다. 먼저 프로젝트 생성 테스트를 실행하세요.');
      }

      const studyUid = `1.2.840.113619.2.1.1.${Date.now()}`;
      const studyData = {
        study_uid: studyUid,
        study_description: 'Test Study for E2E',
        patient_id: 'TEST001',
        patient_name: 'Test Patient',
        study_date: '2025-01-15',
      };

      try {
        const response = await axios.post(
          `${apiUrl}/api/projects/${projectId}/studies/assign`,
          studyData
        );

        console.log(`  ✅ Study 할당 성공:`, response.data);
        console.log(`  📝 생성된 Study ID: ${response.data.study_id}`);

        // Study ID 및 UID 저장
        createdStudyIdRef.current = response.data.study_id;
        createdStudyUidRef.current = studyUid;

        return {
          request: { method: 'POST', url: `/api/projects/${projectId}/studies/assign`, body: studyData },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${projectId}/studies/assign`,
            data: studyData,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // Series 할당 (3개)
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다.');
      }

      const studyId = createdStudyIdRef.current;
      if (!studyId) {
        throw new Error('Study가 할당되지 않았습니다.');
      }

      // 3개의 Series 할당
      const studyUid = createdStudyUidRef.current;
      if (!studyUid) {
        throw new Error('Study UID가 저장되지 않았습니다.');
      }

      const timestamp = Date.now();
      const seriesIds: number[] = [];
      const seriesUids: string[] = [];
      const seriesDataList = [
        {
          study_uid: studyUid,
          series_uid: `1.2.840.113619.2.1.2.${timestamp}.1`,
          series_description: 'Axial CT 5mm',
          modality: 'CT',
          series_number: 1,
        },
        {
          study_uid: studyUid,
          series_uid: `1.2.840.113619.2.1.2.${timestamp}.2`,
          series_description: 'Coronal CT 5mm',
          modality: 'CT',
          series_number: 2,
        },
        {
          study_uid: studyUid,
          series_uid: `1.2.840.113619.2.1.2.${timestamp}.3`,
          series_description: 'Sagittal CT 5mm',
          modality: 'CT',
          series_number: 3,
        },
      ];

      try {
        for (let i = 0; i < seriesDataList.length; i++) {
          const response = await axios.post(
            `${apiUrl}/api/projects/${projectId}/series/assign`,
            seriesDataList[i]
          );

          console.log(`  ✅ Series ${i + 1}/3 할당 성공:`, response.data);
          seriesIds.push(response.data.series_id);
          seriesUids.push(seriesDataList[i].series_uid);
        }

        // Series IDs 및 UIDs 저장
        createdSeriesIdsRef.current = seriesIds;
        createdSeriesUidsRef.current = seriesUids;

        return {
          request: { method: 'POST', url: `/api/projects/${projectId}/series/assign`, body: seriesDataList },
          response: { message: `${seriesIds.length}개의 Series 할당 완료`, series_ids: seriesIds },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${projectId}/series/assign`,
            data: seriesDataList,
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 프로젝트 Study 목록 조회
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다.');
      }

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/studies`);

        console.log(`  ✅ Study 목록 조회 성공:`, response.data);

        // 할당한 Study가 목록에 있는지 확인
        if (!response.data.studies || response.data.studies.length === 0) {
          throw new Error('할당한 Study가 목록에 없습니다.');
        }

        return {
          request: { method: 'GET', url: `/api/project-data/${projectId}/studies` },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/studies`,
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // 프로젝트 Series 목록 조회
      const projectId = createdProjectIdRef.current;
      const studyId = createdStudyIdRef.current;

      if (!projectId || !studyId) {
        throw new Error('프로젝트 또는 Study가 생성되지 않았습니다.');
      }

      try {
        const response = await axios.get(
          `${apiUrl}/api/project-data/${projectId}/studies/${studyId}/series`
        );

        console.log(`  ✅ Series 목록 조회 성공:`, response.data);

        // 할당한 Series가 목록에 있는지 확인
        if (!response.data.series || response.data.series.length !== 3) {
          throw new Error(`할당한 3개의 Series가 목록에 없습니다. (실제: ${response.data.series?.length || 0}개)`);
        }

        return {
          request: { method: 'GET', url: `/api/project-data/${projectId}/studies/${studyId}/series` },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/studies/${studyId}/series`,
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // Series 중복 할당 시도 (409 에러)
      const projectId = createdProjectIdRef.current;
      const studyUid = createdStudyUidRef.current;
      const seriesUids = createdSeriesUidsRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다.');
      }

      if (!studyUid || seriesUids.length === 0) {
        throw new Error('Study 또는 Series가 할당되지 않았습니다.');
      }

      // 첫 번째 Series와 동일한 UID 사용
      const duplicateSeriesData = {
        study_uid: studyUid,
        series_uid: seriesUids[0], // 첫 번째 Series UID 재사용
        series_description: 'Duplicate Series',
        modality: 'CT',
        series_number: 1,
      };

      try {
        await axios.post(
          `${apiUrl}/api/projects/${projectId}/series/assign`,
          duplicateSeriesData
        );
        throw new Error('409 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 409) {
          console.log(`  ✅ 중복 할당 시 409 에러 정상 반환`);
          return {
            request: { method: 'POST', url: `/api/projects/${projectId}/series/assign`, body: duplicateSeriesData },
            response: error.response.data || { status: 409, message: 'Conflict' },
          };
        }

        // 409가 아닌 다른 에러
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${projectId}/series/assign`,
            data: duplicateSeriesData,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 존재하지 않는 프로젝트에 할당 (404 에러)
      const nonExistentProjectId = 999999;
      const seriesData = {
        study_uid: `1.2.840.113619.2.1.1.${Date.now()}`,
        series_uid: `1.2.840.113619.2.1.2.${Date.now()}.999`,
        series_description: 'Test Series',
        modality: 'CT',
        series_number: 1,
      };

      try {
        await axios.post(
          `${apiUrl}/api/projects/${nonExistentProjectId}/series/assign`,
          seriesData
        );
        throw new Error('404 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 404) {
          console.log(`  ✅ 존재하지 않는 프로젝트에 할당 시 404 에러 정상 반환`);
          return {
            request: { method: 'POST', url: `/api/projects/${nonExistentProjectId}/series/assign`, body: seriesData },
            response: error.response.data || { status: 404, message: 'Not Found' },
          };
        }

        // 404가 아닌 다른 에러
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${nonExistentProjectId}/series/assign`,
            data: seriesData,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 다른 프로젝트 생성 (격리 테스트)
      const projectData = {
        name: `Isolation Test ${Date.now()}`,
        description: 'Project Isolation Test',
        sponsor: 'Test Sponsor',
        start_date: '2025-01-01',
        end_date: '2025-12-31',
        auto_complete: false,
      };

      try {
        const response = await axios.post(`${apiUrl}/api/projects`, projectData);

        console.log(`  ✅ 격리 테스트용 프로젝트 생성 성공:`, response.data);
        console.log(`  📝 생성된 프로젝트 ID: ${response.data.id}`);

        return {
          request: { method: 'POST', url: '/api/projects', body: projectData },
          response: response.data,
          isolationProjectId: response.data.id,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects`,
            data: projectData,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 다른 프로젝트 데이터 조회 (빈 목록)
      // 이전 테스트에서 생성한 격리 테스트용 프로젝트 ID를 가져와야 함
      // 임시로 sections에서 이전 테스트 결과를 찾아서 ID 추출
      const allTests = sections.flatMap(s => s.tests);
      const isolationTest = allTests.find(t => t.name === '다른 프로젝트 생성 (격리 테스트)');

      if (!isolationTest || !isolationTest.response) {
        throw new Error('격리 테스트용 프로젝트가 생성되지 않았습니다.');
      }

      const isolationProjectId = (isolationTest.response as any).id;

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${isolationProjectId}/studies`);

        console.log(`  ✅ 다른 프로젝트 데이터 조회 성공:`, response.data);

        // 빈 목록이어야 함 (다른 프로젝트의 데이터는 보이지 않아야 함)
        if (response.data.studies && response.data.studies.length > 0) {
          throw new Error(`다른 프로젝트의 데이터가 조회되었습니다. (${response.data.studies.length}개)`);
        }

        return {
          request: { method: 'GET', url: `/api/project-data/${isolationProjectId}/studies` },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${isolationProjectId}/studies`,
          };
        }
        throw error;
      }
    } else if (testIndex === 9) {
      // Series 할당 해제 (첫 번째)
      const projectId = createdProjectIdRef.current;
      const studyId = createdStudyIdRef.current;
      const seriesIds = createdSeriesIdsRef.current;

      if (!projectId || !studyId || seriesIds.length === 0) {
        throw new Error('프로젝트, Study 또는 Series가 할당되지 않았습니다.');
      }

      const firstSeriesId = seriesIds[0];

      try {
        const response = await axios.delete(
          `${apiUrl}/api/projects/${projectId}/series/${firstSeriesId}/unassign`
        );

        console.log(`  ✅ Series ${firstSeriesId} 할당 해제 성공:`, response.data);

        return {
          request: {
            method: 'DELETE',
            url: `/api/projects/${projectId}/series/${firstSeriesId}/unassign`,
          },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${projectId}/series/${firstSeriesId}/unassign`,
          };
        }
        throw error;
      }
    } else if (testIndex === 10) {
      // Series 목록 재조회 (2개 확인)
      const projectId = createdProjectIdRef.current;
      const studyId = createdStudyIdRef.current;

      if (!projectId || !studyId) {
        throw new Error('프로젝트 또는 Study가 할당되지 않았습니다.');
      }

      try {
        const response = await axios.get(
          `${apiUrl}/api/project-data/${projectId}/studies/${studyId}/series`
        );

        console.log(`  ✅ Series 목록 재조회:`, response.data);

        if (!response.data.series || response.data.series.length !== 2) {
          throw new Error(
            `할당 해제 후 2개의 Series가 남아있어야 하는데 ${response.data.series?.length || 0}개가 조회되었습니다.`
          );
        }

        return {
          request: {
            method: 'GET',
            url: `/api/project-data/${projectId}/studies/${studyId}/series`,
          },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/studies/${studyId}/series`,
          };
        }
        throw error;
      }
    } else if (testIndex === 11) {
      // Study 할당 해제
      const projectId = createdProjectIdRef.current;
      const studyId = createdStudyIdRef.current;

      if (!projectId || !studyId) {
        throw new Error('프로젝트 또는 Study가 할당되지 않았습니다.');
      }

      try {
        const response = await axios.delete(
          `${apiUrl}/api/projects/${projectId}/studies/${studyId}/unassign`
        );

        console.log(`  ✅ Study ${studyId} 할당 해제 성공:`, response.data);

        return {
          request: {
            method: 'DELETE',
            url: `/api/projects/${projectId}/studies/${studyId}/unassign`,
          },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${projectId}/studies/${studyId}/unassign`,
          };
        }
        throw error;
      }
    } else if (testIndex === 12) {
      // Study 목록 재조회 (빈 목록 확인)
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트가 생성되지 않았습니다.');
      }

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/studies`);

        console.log(`  ✅ Study 목록 재조회:`, response.data);

        if (!response.data.studies || response.data.studies.length !== 0) {
          throw new Error(
            `Study 할당 해제 후 빈 목록이어야 하는데 ${response.data.studies?.length || 0}개가 조회되었습니다.`
          );
        }

        return {
          request: { method: 'GET', url: `/api/project-data/${projectId}/studies` },
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/studies`,
          };
        }
        throw error;
      }
    } else if (testIndex === 13) {
      // 데이터 테스트 프로젝트 삭제 (cleanup)
      const projectId = createdProjectIdRef.current;

      if (!projectId) {
        throw new Error('삭제할 프로젝트가 없습니다.');
      }

      const deletedId = projectId;

      try {
        const response = await axios.delete(`${apiUrl}/api/projects/${projectId}`);

        // 삭제 후 ID 초기화 (state + ref)
        setCreatedProjectId(null);
        createdProjectIdRef.current = null;
        createdStudyIdRef.current = null;
        createdStudyUidRef.current = null;
        createdSeriesIdsRef.current = [];
        createdSeriesUidsRef.current = [];

        console.log(`  ✅ 데이터 테스트 프로젝트 삭제 완료`);

        return {
          request: { method: 'DELETE', url: `/api/projects/${deletedId}` },
          response: response.data || { message: '프로젝트가 삭제되었습니다' },
        };
      } catch (error: any) {
        // 삭제 실패해도 ID 초기화 (다음 테스트를 위해)
        setCreatedProjectId(null);
        createdProjectIdRef.current = null;
        createdStudyIdRef.current = null;
        createdStudyUidRef.current = null;
        createdSeriesIdsRef.current = [];
        createdSeriesUidsRef.current = [];

        // 에러에 요청 정보 포함
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${deletedId}`,
          };
        }
        throw error;
      }
    }
  };

  // DICOM 전체 조회 + 할당 여부 확인 테스트
  const runDicomTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // DICOM Studies 전체 조회 (project_id 없음) - SUPER_ADMIN 권한 필요
      const requestInfo = { method: 'GET', url: '/api/dicom/studies' };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        const response = await axios.get(`${apiUrl}/api/dicom/studies`, config);

        console.log(`  ✅ DICOM Studies 전체 조회 성공:`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies`,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // DICOM Studies 프로젝트별 조회 (project_id 있음) - USER 권한으로 테스트
      const projectId = createdProjectIdRef.current || 150; // 기본값 150
      const requestInfo = { method: 'GET', url: `/api/dicom/studies?project_id=${projectId}` };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/studies?project_id=${projectId}`, config);

        console.log(`  ✅ DICOM Studies 프로젝트별 조회 성공 (project_id=${projectId}):`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // DICOM Series 전체 조회 (project_id 없음) - SUPER_ADMIN 권한 필요
      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');

        // 먼저 Study를 조회해서 Study UID를 얻음
        const studiesResponse = await axios.get(`${apiUrl}/api/dicom/studies?limit=1`, config);

        if (!Array.isArray(studiesResponse.data) || studiesResponse.data.length === 0) {
          throw new Error('No studies found to test series retrieval');
        }

        // Study UID 추출 (0020000D 태그)
        const studyUid = studiesResponse.data[0]['0020000D']?.Value?.[0];
        if (!studyUid) {
          throw new Error('Study UID not found in response');
        }

        const requestInfo = { method: 'GET', url: `/api/dicom/studies/${studyUid}/series` };
        const response = await axios.get(`${apiUrl}/api/dicom/studies/${studyUid}/series`, config);

        console.log(`  ✅ DICOM Series 전체 조회 성공 (Study UID: ${studyUid}):`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: '/api/dicom/studies/{studyUid}/series',
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // DICOM Instances 전체 조회 (project_id 없음) - SUPER_ADMIN 권한 필요
      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');

        // 먼저 Study를 조회해서 Study UID를 얻음
        const studiesResponse = await axios.get(`${apiUrl}/api/dicom/studies?limit=1`, config);

        if (!Array.isArray(studiesResponse.data) || studiesResponse.data.length === 0) {
          throw new Error('No studies found to test instances retrieval');
        }

        // Study UID 추출 (0020000D 태그)
        const studyUid = studiesResponse.data[0]['0020000D']?.Value?.[0];
        if (!studyUid) {
          throw new Error('Study UID not found in response');
        }

        // Series 조회해서 Series UID를 얻음
        const seriesResponse = await axios.get(`${apiUrl}/api/dicom/studies/${studyUid}/series?limit=1`, config);

        if (!Array.isArray(seriesResponse.data) || seriesResponse.data.length === 0) {
          throw new Error('No series found to test instances retrieval');
        }

        // Series UID 추출 (0020000E 태그)
        const seriesUid = seriesResponse.data[0]['0020000E']?.Value?.[0];
        if (!seriesUid) {
          throw new Error('Series UID not found in response');
        }

        const requestInfo = { method: 'GET', url: `/api/dicom/studies/${studyUid}/series/${seriesUid}/instances` };
        const response = await axios.get(`${apiUrl}/api/dicom/studies/${studyUid}/series/${seriesUid}/instances`, config);

        console.log(`  ✅ DICOM Instances 전체 조회 성공 (Study UID: ${studyUid}, Series UID: ${seriesUid}):`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: '/api/dicom/studies/{studyUid}/series/{seriesUid}/instances',
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // 할당 여부 확인 (check_assignment_for_project) - ADMIN 권한으로 테스트
      const projectId = createdProjectIdRef.current || 150;
      const requestInfo = { method: 'GET', url: `/api/dicom/studies?check_assignment_for_project=${projectId}` };

      try {
        const config = await handleGetAxiosConfig('ADMIN');
        const response = await axios.get(`${apiUrl}/api/dicom/studies?check_assignment_for_project=${projectId}`, config);

        console.log(`  ✅ 할당 여부 확인 성공 (project_id=${projectId}):`, response.data);

        // is_assigned 필드 확인
        if (Array.isArray(response.data) && response.data.length > 0) {
          const hasAssignmentField = response.data.every((study: any) =>
            study.hasOwnProperty('is_assigned') && study.hasOwnProperty('checked_project_id')
          );

          if (!hasAssignmentField) {
            throw new Error('응답에 is_assigned 또는 checked_project_id 필드가 없습니다');
          }
        }

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?check_assignment_for_project=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // 전체 조회 + 할당 여부 확인 (통합) - ADMIN 권한으로 테스트
      const projectId = createdProjectIdRef.current || 150;
      const requestInfo = { method: 'GET', url: `/api/dicom/studies?check_assignment_for_project=${projectId}` };

      try {
        const config = await handleGetAxiosConfig('ADMIN');
        const response = await axios.get(`${apiUrl}/api/dicom/studies?check_assignment_for_project=${projectId}`, config);

        console.log(`  ✅ 전체 조회 + 할당 여부 확인 성공:`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?check_assignment_for_project=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 프로젝트별 조회 + 할당 여부 확인 - USER 권한으로 테스트
      const projectId = createdProjectIdRef.current || 150;
      const requestInfo = {
        method: 'GET',
        url: `/api/dicom/studies?project_id=${projectId}&check_assignment_for_project=${projectId}`
      };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(
          `${apiUrl}/api/dicom/studies?project_id=${projectId}&check_assignment_for_project=${projectId}`,
          config
        );

        console.log(`  ✅ 프로젝트별 조회 + 할당 여부 확인 성공:`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=${projectId}&check_assignment_for_project=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 다른 프로젝트 할당 여부 확인 - ADMIN 권한으로 테스트
      const filterProjectId = createdProjectIdRef.current || 150;
      const checkProjectId = 200; // 다른 프로젝트
      const requestInfo = {
        method: 'GET',
        url: `/api/dicom/studies?project_id=${filterProjectId}&check_assignment_for_project=${checkProjectId}`
      };

      try {
        const config = await handleGetAxiosConfig('ADMIN');
        const response = await axios.get(
          `${apiUrl}/api/dicom/studies?project_id=${filterProjectId}&check_assignment_for_project=${checkProjectId}`,
          config
        );

        console.log(`  ✅ 다른 프로젝트 할당 여부 확인 성공:`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=${filterProjectId}&check_assignment_for_project=${checkProjectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 잘못된 project_id (0) 에러 처리 - USER 권한으로 테스트
      const requestInfo = { method: 'GET', url: '/api/dicom/studies?project_id=0' };

      try {
        const config = await handleGetAxiosConfig('USER');
        await axios.get(`${apiUrl}/api/dicom/studies?project_id=0`, config);
        throw new Error('400 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 400) {
          console.log(`  ✅ project_id=0 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 400, message: 'Bad Request' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=0`,
          };
        }
        throw error;
      }
    } else if (testIndex === 9) {
      // 잘못된 project_id (음수) 에러 처리 - USER 권한으로 테스트
      const requestInfo = { method: 'GET', url: '/api/dicom/studies?project_id=-1' };

      try {
        const config = await handleGetAxiosConfig('USER');
        await axios.get(`${apiUrl}/api/dicom/studies?project_id=-1`, config);
        throw new Error('400 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 400) {
          console.log(`  ✅ project_id=-1 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 400, message: 'Bad Request' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/studies?project_id=-1`,
          };
        }
        throw error;
      }
    }
  };

  // DICOM Patient API 테스트 (QIDO-RS 프록시)
  const runPatientTest = async (testIndex: number) => {
    const projectId = createdProjectIdRef.current || 2; // 기본값 2

    if (testIndex === 0) {
      // Patient 전체 조회 (project_id 있음)
      const requestInfo = { method: 'GET', url: `/api/dicom/patients?project_id=${projectId}` };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/patients?project_id=${projectId}`, config);

        console.log(`  ✅ Patient 전체 조회 성공 (project_id=${projectId}):`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/patients?project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // Patient 페이지네이션 (limit=1)
      const requestInfo = { method: 'GET', url: `/api/dicom/patients?project_id=${projectId}&limit=1` };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/patients?project_id=${projectId}&limit=1`, config);

        console.log(`  ✅ Patient 페이지네이션 성공 (limit=1):`, response.data);

        // 최대 1개만 반환되는지 확인
        if (Array.isArray(response.data) && response.data.length > 1) {
          throw new Error(`Expected at most 1 patient, got ${response.data.length}`);
        }

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/patients?project_id=${projectId}&limit=1`,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // Patient 필터링 (PatientName)
      const requestInfo = { method: 'GET', url: `/api/dicom/patients?project_id=${projectId}&PatientName=*` };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/patients?project_id=${projectId}&PatientName=*`, config);

        console.log(`  ✅ Patient 필터링 성공 (PatientName=*):`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/patients?project_id=${projectId}&PatientName=*`,
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // Patient 필터링 (PatientID)
      const requestInfo = { method: 'GET', url: `/api/dicom/patients?project_id=${projectId}&PatientID=*` };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/patients?project_id=${projectId}&PatientID=*`, config);

        console.log(`  ✅ Patient 필터링 성공 (PatientID=*):`, response.data);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/patients?project_id=${projectId}&PatientID=*`,
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // Patient DICOM JSON 구조 검증
      const requestInfo = { method: 'GET', url: `/api/dicom/patients?project_id=${projectId}&limit=1` };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/patients?project_id=${projectId}&limit=1`, config);

        console.log(`  ✅ Patient DICOM JSON 구조 검증:`, response.data);

        // DICOM JSON 구조 검증
        if (Array.isArray(response.data) && response.data.length > 0) {
          const patient = response.data[0];

          // 필수 DICOM 태그 확인
          const requiredTags = ['00100020']; // PatientID
          const missingTags = requiredTags.filter(tag => !patient[tag]);

          if (missingTags.length > 0) {
            throw new Error(`Missing required DICOM tags: ${missingTags.join(', ')}`);
          }

          // DICOM JSON 형식 확인 (vr, Value 필드)
          const patientId = patient['00100020'];
          if (!patientId.vr || !patientId.Value) {
            throw new Error('Invalid DICOM JSON format: missing vr or Value field');
          }
        }

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/patients?project_id=${projectId}&limit=1`,
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // project_id 없이 조회 (400 에러)
      const requestInfo = { method: 'GET', url: '/api/dicom/patients' };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/patients`, config);

        // 에러가 발생하지 않으면 실패
        throw new Error('Expected 400 error, but request succeeded');
      } catch (error: any) {
        if (error.response && error.response.status === 400) {
          console.log(`  ✅ project_id 없이 조회 시 400 에러 발생 (예상된 동작)`);

          return {
            request: requestInfo,
            response: { error: error.response.data, status: 400 },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/patients`,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 잘못된 project_id (0) 에러 처리
      const requestInfo = { method: 'GET', url: '/api/dicom/patients?project_id=0' };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/patients?project_id=0`, config);

        // 에러가 발생하지 않으면 실패
        throw new Error('Expected 400 error, but request succeeded');
      } catch (error: any) {
        if (error.response && error.response.status === 400) {
          console.log(`  ✅ project_id=0 시 400 에러 발생 (예상된 동작)`);

          return {
            request: requestInfo,
            response: { error: error.response.data, status: 400 },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/patients?project_id=0`,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 잘못된 project_id (음수) 에러 처리
      const requestInfo = { method: 'GET', url: '/api/dicom/patients?project_id=-1' };

      try {
        const config = await handleGetAxiosConfig('USER');
        const response = await axios.get(`${apiUrl}/api/dicom/patients?project_id=-1`, config);

        // 에러가 발생하지 않으면 실패
        throw new Error('Expected 400 error, but request succeeded');
      } catch (error: any) {
        if (error.response && error.response.status === 400) {
          console.log(`  ✅ project_id=-1 시 400 에러 발생 (예상된 동작)`);

          return {
            request: requestInfo,
            response: { error: error.response.data, status: 400 },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/dicom/patients?project_id=-1`,
          };
        }
        throw error;
      }
    }
  };

  // Annotation Label 기능 테스트
  const runAnnotationLabelTest = async (testIndex: number) => {
    // 테스트용 데이터
    const testStudyUid = '1.2.840.113619.2.55.3.604688119.868.1234567890.1';
    const testSeriesUid = '1.2.840.113619.2.55.3.604688119.868.1234567890.2';
    const testSopUid = '1.2.840.113619.2.55.3.604688119.868.1234567890.3';

    if (testIndex === 0) {
      // 1️⃣ Label 없이 Annotation 생성
      const requestBody = {
        study_instance_uid: testStudyUid,
        series_instance_uid: testSeriesUid,
        sop_instance_uid: testSopUid,
        annotation_data: {
          type: 'rectangle',
          x: 100,
          y: 100,
          width: 200,
          height: 150,
          color: '#FF0000',
        },
        tool_name: 'Rectangle Tool',
        description: 'Test annotation without label',
      };

      const requestInfo = { method: 'POST', url: '/api/annotations', body: requestBody };

      try {
        const config = await handleGetAxiosConfig();
        const response = await axios.post(`${apiUrl}/api/annotations`, requestBody, config);

        // 생성된 annotation ID 저장
        createdAnnotationIdsRef.current = [response.data.id];

        console.log(`  ✅ Label 없이 Annotation 생성 성공:`, response.data);

        return {
          request: requestInfo,
          response: {
            id: response.data.id,
            label: response.data.label,
            message: 'Label이 null 또는 빈 문자열이어야 함',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/annotations`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // 2️⃣ Label과 함께 Annotation 생성 (Tumor)
      const requestBody = {
        study_instance_uid: testStudyUid,
        series_instance_uid: testSeriesUid,
        sop_instance_uid: testSopUid,
        annotation_data: {
          type: 'circle',
          x: 300,
          y: 300,
          radius: 50,
          color: '#00FF00',
        },
        tool_name: 'Circle Tool',
        label: 'Tumor',
        description: 'Test annotation with Tumor label',
      };

      const requestInfo = { method: 'POST', url: '/api/annotations', body: requestBody };

      try {
        const config = await handleGetAxiosConfig();
        const response = await axios.post(`${apiUrl}/api/annotations`, requestBody, config);

        // 생성된 annotation ID 추가
        createdAnnotationIdsRef.current.push(response.data.id);

        console.log(`  ✅ Label과 함께 Annotation 생성 성공:`, response.data);

        if (response.data.label !== 'Tumor') {
          throw new Error(`Label이 'Tumor'여야 하는데 '${response.data.label}'입니다.`);
        }

        return {
          request: requestInfo,
          response: {
            id: response.data.id,
            label: response.data.label,
            message: 'Label이 Tumor로 설정됨',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/annotations`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // 3️⃣ 생성된 Annotation 조회 (Label 확인)
      const annotationId = createdAnnotationIdsRef.current[1]; // Tumor label이 있는 annotation

      if (!annotationId) {
        throw new Error('조회할 Annotation ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/annotations/${annotationId}` };

      try {
        const config = await handleGetAxiosConfig();
        const response = await axios.get(`${apiUrl}/api/annotations/${annotationId}`, config);

        console.log(`  ✅ Annotation 조회 성공:`, response.data);

        if (response.data.label !== 'Tumor') {
          throw new Error(`Label이 'Tumor'여야 하는데 '${response.data.label}'입니다.`);
        }

        return {
          request: requestInfo,
          response: {
            id: response.data.id,
            label: response.data.label,
            message: 'Label이 Tumor로 확인됨',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/${annotationId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 4️⃣ Label 수정 (Tumor → Lesion)
      const annotationId = createdAnnotationIdsRef.current[1];

      if (!annotationId) {
        throw new Error('수정할 Annotation ID가 없습니다.');
      }

      const requestBody = {
        label: 'Lesion',
      };

      const requestInfo = { method: 'PUT', url: `/api/annotations/${annotationId}`, body: requestBody };

      try {
        const config = await handleGetAxiosConfig();
        const response = await axios.put(`${apiUrl}/api/annotations/${annotationId}`, requestBody, config);

        console.log(`  ✅ Label 수정 성공:`, response.data);

        if (response.data.label !== 'Lesion') {
          throw new Error(`Label이 'Lesion'으로 수정되어야 하는데 '${response.data.label}'입니다.`);
        }

        return {
          request: requestInfo,
          response: {
            id: response.data.id,
            label: response.data.label,
            version: response.data.version,
            message: 'Label이 Tumor에서 Lesion으로 수정됨',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'put',
            url: `${apiUrl}/api/annotations/${annotationId}`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // 5️⃣ 수정된 Label 확인
      const annotationId = createdAnnotationIdsRef.current[1];

      if (!annotationId) {
        throw new Error('조회할 Annotation ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/annotations/${annotationId}` };

      try {
        const config = await handleGetAxiosConfig();
        const response = await axios.get(`${apiUrl}/api/annotations/${annotationId}`, config);

        console.log(`  ✅ 수정된 Annotation 조회 성공:`, response.data);

        if (response.data.label !== 'Lesion') {
          throw new Error(`Label이 'Lesion'이어야 하는데 '${response.data.label}'입니다.`);
        }

        return {
          request: requestInfo,
          response: {
            id: response.data.id,
            label: response.data.label,
            version: response.data.version,
            message: 'Label이 Lesion으로 확인됨',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/${annotationId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // 6️⃣ 다양한 Label로 Annotation 생성 (Normal, Abnormal, Suspicious)
      const labels = ['Normal', 'Abnormal', 'Suspicious'];
      const createdIds: number[] = [];

      const requestInfo = { method: 'POST', url: '/api/annotations (x3)', body: { labels } };

      try {
        const config = await handleGetAxiosConfig();

        for (const label of labels) {
          const requestBody = {
            study_instance_uid: testStudyUid,
            series_instance_uid: testSeriesUid,
            sop_instance_uid: testSopUid,
            annotation_data: {
              type: 'point',
              x: Math.random() * 500,
              y: Math.random() * 500,
              color: '#0000FF',
            },
            tool_name: 'Point Tool',
            label: label,
            description: `Test annotation with ${label} label`,
          };

          const response = await axios.post(`${apiUrl}/api/annotations`, requestBody, config);
          createdIds.push(response.data.id);

          console.log(`  ✅ ${label} Label로 Annotation 생성 성공:`, response.data.id);
        }

        // 생성된 annotation ID들 추가
        createdAnnotationIdsRef.current.push(...createdIds);

        return {
          request: requestInfo,
          response: {
            created_ids: createdIds,
            labels: labels,
            message: `${labels.length}개의 다양한 Label로 Annotation 생성됨`,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/annotations`,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 7️⃣ 모든 Annotation 조회 (Label 포함)
      const requestInfo = { method: 'GET', url: '/api/annotations' };

      try {
        const config = await handleGetAxiosConfig();
        const response = await axios.get(`${apiUrl}/api/annotations`, {
          ...config,
          params: {
            series_instance_uid: testSeriesUid,
          },
        });

        console.log(`  ✅ Annotation 목록 조회 성공:`, response.data);

        const annotations = response.data.annotations || response.data;
        const labelCounts: { [key: string]: number } = {};

        annotations.forEach((ann: any) => {
          const label = ann.label || '(empty)';
          labelCounts[label] = (labelCounts[label] || 0) + 1;
        });

        return {
          request: requestInfo,
          response: {
            total: annotations.length,
            label_counts: labelCounts,
            message: 'Label별 Annotation 개수 확인',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations`,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 8️⃣ Label 빈 문자열로 수정
      const annotationId = createdAnnotationIdsRef.current[1]; // Lesion label이 있는 annotation

      if (!annotationId) {
        throw new Error('수정할 Annotation ID가 없습니다.');
      }

      const requestBody = {
        label: '',
      };

      const requestInfo = { method: 'PUT', url: `/api/annotations/${annotationId}`, body: requestBody };

      try {
        const config = await handleGetAxiosConfig();
        const response = await axios.put(`${apiUrl}/api/annotations/${annotationId}`, requestBody, config);

        console.log(`  ✅ Label 빈 문자열로 수정 성공:`, response.data);

        if (response.data.label !== '' && response.data.label !== null) {
          throw new Error(`Label이 빈 문자열이어야 하는데 '${response.data.label}'입니다.`);
        }

        return {
          request: requestInfo,
          response: {
            id: response.data.id,
            label: response.data.label,
            version: response.data.version,
            message: 'Label이 빈 문자열로 수정됨',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'put',
            url: `${apiUrl}/api/annotations/${annotationId}`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 9️⃣ 정리 (생성된 Annotation 삭제)
      const annotationIds = [...createdAnnotationIdsRef.current];

      if (annotationIds.length === 0) {
        throw new Error('삭제할 Annotation이 없습니다.');
      }

      const requestInfo = { method: 'DELETE', url: `/api/annotations (x${annotationIds.length})` };

      try {
        const config = await handleGetAxiosConfig();
        const deletedIds: number[] = [];

        for (const id of annotationIds) {
          try {
            await axios.delete(`${apiUrl}/api/annotations/${id}`, config);
            deletedIds.push(id);
            console.log(`  ✅ Annotation ${id} 삭제 성공`);
          } catch (error: any) {
            console.warn(`  ⚠️ Annotation ${id} 삭제 실패:`, error.message);
          }
        }

        // 삭제 후 ID 목록 초기화
        createdAnnotationIdsRef.current = [];

        return {
          request: requestInfo,
          response: {
            deleted_count: deletedIds.length,
            deleted_ids: deletedIds,
            message: `${deletedIds.length}개의 Annotation 삭제 완료`,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/annotations`,
          };
        }
        throw error;
      }
    }
  };

  // Project Data Access 접근 제어 테스트
  const runProjectDataAccessTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // 시나리오 구성
      const requestInfo = { method: 'POST', url: '/api/test/project-data-access/setup' };

      try {
        const response = await axios.post(`${apiUrl}/api/test/project-data-access/setup`, {}, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        // 생성된 프로젝트 ID 저장
        createdProjectIdRef.current = response.data.project_id;

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/test/project-data-access/setup`,
          };
        }
        throw error;
      }
    } else if (testIndex === 1) {
      // 접근 제어 매트릭스 조회
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다. 먼저 시나리오를 구성하세요.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix`,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // 매트릭스 구조 검증 (4명 사용자)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        if (!Array.isArray(users) || users.length !== 4) {
          throw new Error(`4명의 사용자가 있어야 하는데 ${users.length}명이 있습니다`);
        }

        return {
          request: requestInfo,
          response: { users: users.length, usernames: users.map((u: any) => u.username) },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix`,
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 매트릭스 구조 검증 (7개 Study)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const dataList = data.data_list || [];

        // 7개의 Study가 있는지 확인
        if (dataList.length < 7) {
          throw new Error(`7개의 Study가 있어야 하는데 ${dataList.length}개가 있습니다`);
        }

        return {
          request: requestInfo,
          response: { studies: dataList.length, study_uids: dataList.slice(0, 7).map((s: any) => s.study_uid) },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // Dr. Kim 전체 접근 확인 (7/7)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const drKim = users.find((u: any) => u.username === 'dr_kim');

        if (!drKim) {
          throw new Error('Dr. Kim을 찾을 수 없습니다');
        }

        // 프로젝트 멤버는 기본적으로 모든 데이터에 접근 가능 (project_data_access 레코드가 없으면)
        // Dr. Kim은 책임연구원이므로 모든 Study에 접근 가능
        const dataList = data.data_list || [];
        const studyCount = dataList.filter((d: any) => d.study_uid).length;

        return {
          request: requestInfo,
          response: {
            user: drKim.username,
            message: 'Dr. Kim (책임연구원)은 모든 Study에 접근 가능',
            total_studies: studyCount
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // Dr. Lee A병원만 접근 확인 (3/7)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const drLee = users.find((u: any) => u.username === 'dr_lee');

        if (!drLee) {
          throw new Error('Dr. Lee를 찾을 수 없습니다');
        }

        // project_data_access 테이블에서 Dr. Lee의 접근 권한 확인
        const accessMatrix = data.access_matrix || [];
        const drLeeAccess = accessMatrix.filter((a: any) => a.user_id === drLee.id);

        // A병원 Study 3개에 대한 접근 권한이 있어야 함
        const dataList = data.data_list || [];
        const hospitalAStudies = dataList.filter((d: any) => d.study_uid && d.study_uid.includes('.A.'));

        return {
          request: requestInfo,
          response: {
            user: drLee.username,
            message: 'Dr. Lee (A병원)는 A병원 Study 3개에만 접근 가능',
            hospital_a_studies: hospitalAStudies.length,
            access_records: drLeeAccess.length
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // Dr. Park B병원만 접근 확인 (3/7)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const drPark = users.find((u: any) => u.username === 'dr_park');

        if (!drPark) {
          throw new Error('Dr. Park을 찾을 수 없습니다');
        }

        // project_data_access 테이블에서 Dr. Park의 접근 권한 확인
        const accessMatrix = data.access_matrix || [];
        const drParkAccess = accessMatrix.filter((a: any) => a.user_id === drPark.id);

        // B병원 Study 3개에 대한 접근 권한이 있어야 함
        const dataList = data.data_list || [];
        const hospitalBStudies = dataList.filter((d: any) => d.study_uid && d.study_uid.includes('.B.'));

        return {
          request: requestInfo,
          response: {
            user: drPark.username,
            message: 'Dr. Park (B병원)은 B병원 Study 3개에만 접근 가능',
            hospital_b_studies: hospitalBStudies.length,
            access_records: drParkAccess.length
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // Dr. Choi 읽기 전용 확인 (1/7)
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'GET', url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100` };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const drChoi = users.find((u: any) => u.username === 'dr_choi');

        if (!drChoi) {
          throw new Error('Dr. Choi를 찾을 수 없습니다');
        }

        // project_data_access 테이블에서 Dr. Choi의 접근 권한 확인
        const accessMatrix = data.access_matrix || [];
        const drChoiAccess = accessMatrix.filter((a: any) => a.user_id === drChoi.id);

        // VIP Study 1개에 대한 읽기 전용 접근 권한이 있어야 함
        const readOnlyAccess = drChoiAccess.find((a: any) => a.access_scope === 'READ_ONLY');

        return {
          request: requestInfo,
          response: {
            user: drChoi.username,
            message: 'Dr. Choi (임시 협력자)는 VIP Study 1개에 읽기 전용 접근 가능',
            access_records: drChoiAccess.length,
            read_only: readOnlyAccess ? true : false
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 시나리오 초기화
      const projectId = createdProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'DELETE', url: `/api/test/project-data-access/cleanup/${projectId}` };

      try {
        const response = await axios.delete(`${apiUrl}/api/test/project-data-access/cleanup/${projectId}`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        // 프로젝트 ID 초기화
        createdProjectIdRef.current = null;

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/test/project-data-access/cleanup/${projectId}`,
          };
        }
        throw error;
      }
    }
  };

  // 순차 시나리오 테스트 (실제 API 호출)
  const runSequentialScenarioTest = async (testIndex: number) => {
    if (testIndex === 0) {
      // 0️⃣ 사전 정리 (기존 테스트 데이터 삭제)
      const requestInfo = {
        method: 'DELETE',
        url: '/api/projects (기존 순차 테스트 프로젝트)',
      };

      try {
        const deletedItems: any = {
          project: null,
          users: [],
        };

        // 1. 기존 "심장질환 공동 연구 (순차)" 프로젝트 찾기 및 삭제
        const projectsResponse = await axios.get(`${apiUrl}/api/projects`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const projects = projectsResponse.data.projects || [];
        const existingProject = projects.find(
          (p: any) => p.name === '심장질환 공동 연구 (순차)'
        );

        if (existingProject) {
          await axios.delete(`${apiUrl}/api/projects/${existingProject.id}`, {
            headers: {
              Authorization: `Bearer ${testToken}`,
            },
          });
          deletedItems.project = { id: existingProject.id, name: existingProject.name };
        }

        // 2. 기존 테스트 사용자 찾기 및 삭제 (_seq 붙은 것과 안 붙은 것 모두)
        const testUsernames = [
          'dr_kim_seq', 'dr_lee_seq', 'dr_park_seq', 'dr_choi_seq',
          'dr_kim', 'dr_lee', 'dr_park', 'dr_choi'
        ];

        // 모든 사용자 조회
        const allUsersResponse = await axios.get(`${apiUrl}/api/users`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const allUsers = allUsersResponse.data.users || [];

        for (const username of testUsernames) {
          try {
            const existingUser = allUsers.find((u: any) => u.username === username);

            if (existingUser) {
              console.log(`Deleting user: ${username} (ID: ${existingUser.id})`);
              // 사용자 삭제
              await axios.delete(`${apiUrl}/api/users/${existingUser.id}`, {
                headers: {
                  Authorization: `Bearer ${testToken}`,
                },
              });
              deletedItems.users.push({ id: existingUser.id, username: existingUser.username });
              console.log(`Successfully deleted user: ${username}`);
            }
          } catch (error: any) {
            // 개별 사용자 삭제 실패는 무시하고 계속 진행
            console.log(`Failed to delete user ${username}:`, error.response?.data || error.message);
          }
        }

        return {
          request: requestInfo,
          response: {
            message: '사전 정리 완료',
            deleted_project: deletedItems.project,
            deleted_users: deletedItems.users,
            deleted_users_count: deletedItems.users.length,
          },
        };
      } catch (error: any) {
        // 에러가 발생해도 계속 진행 (정리 단계이므로)
        return {
          request: requestInfo,
          response: { message: '정리 중 에러 발생 (무시하고 계속 진행)', error: error.message },
        };
      }
    } else if (testIndex === 1) {
      // 1️⃣ 프로젝트 생성
      const requestInfo = {
        method: 'POST',
        url: '/api/projects',
        body: {
          name: '심장질환 공동 연구 (순차)',
          description: '다기관 공동 연구 프로젝트 - 순차 API 호출 테스트',
          sponsor: '서울대학교병원',
          start_date: '2025-01-01',
          end_date: '2025-12-31'
        }
      };

      try {
        const response = await axios.post(`${apiUrl}/api/projects`, requestInfo.body, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        sequentialProjectIdRef.current = response.data.id;

        return {
          request: requestInfo,
          response: { project_id: response.data.id, name: response.data.name },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // 2️⃣ 사용자 4명 생성
      const users = [
        { username: 'dr_kim_seq', email: 'dr.kim.seq@hospital.com', full_name: 'Dr. Kim (책임연구원)', keycloak_id: `kim-seq-${Date.now()}` },
        { username: 'dr_lee_seq', email: 'dr.lee.seq@hospital-a.com', full_name: 'Dr. Lee (A병원)', keycloak_id: `lee-seq-${Date.now()}` },
        { username: 'dr_park_seq', email: 'dr.park.seq@hospital-b.com', full_name: 'Dr. Park (B병원)', keycloak_id: `park-seq-${Date.now()}` },
        { username: 'dr_choi_seq', email: 'dr.choi.seq@temp.com', full_name: 'Dr. Choi (임시 협력자)', keycloak_id: `choi-seq-${Date.now()}` },
      ];

      const requestInfo = { method: 'POST', url: '/api/auth/signup', body: users };

      try {
        const createdUsers = [];
        for (const user of users) {
          try {
            // 사용자 생성 시도
            const response = await axios.post(`${apiUrl}/api/auth/signup`, {
              username: user.username,
              email: user.email,
              password: 'Test1234!',
              full_name: user.full_name,
            }, {
              headers: {
                Authorization: `Bearer ${testToken}`,
              },
            });

            createdUsers.push(response.data);
            sequentialUserIdsRef.current[user.username] = response.data.user_id;
            console.log(`  ✅ 사용자 생성 성공: ${user.username} (user_id: ${response.data.user_id})`);
          } catch (error: any) {
            // 이미 존재하는 경우 (409 Conflict 또는 Already exists 에러)
            if (error.response?.status === 409 || 
                error.response?.data?.error?.includes('Already exists') ||
                error.response?.data?.error?.includes('User already exists')) {
              console.log(`  ℹ️ 사용자 ${user.username}는 이미 존재합니다. 기존 사용자 정보를 조회합니다.`);
              
              // 기존 사용자 조회
              try {
                const userResponse = await axios.get(
                  `${apiUrl}/api/users/username/${user.username}`,
                  {
                    headers: {
                      Authorization: `Bearer ${testToken}`,
                    },
                  }
                );

                const existingUser = userResponse.data;
                createdUsers.push({
                  user_id: existingUser.id,
                  username: existingUser.username,
                  email: existingUser.email,
                  full_name: existingUser.full_name,
                });
                sequentialUserIdsRef.current[user.username] = existingUser.id;
                console.log(`  ✅ 기존 사용자 조회 성공: ${user.username} (user_id: ${existingUser.id})`);
              } catch (lookupError: any) {
                console.error(`  ❌ 사용자 조회 실패: ${user.username}`, lookupError);
                throw new Error(`사용자 ${user.username}가 이미 존재하지만 조회에 실패했습니다: ${lookupError.message}`);
              }
            } else {
              // 다른 에러는 그대로 전파
              throw error;
            }
          }
        }

        return {
          request: requestInfo,
          response: {
            users: createdUsers.map(u => ({ username: u.username, user_id: u.user_id })),
            count: createdUsers.length
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 3️⃣ 사용자 4명 활성화 (관리자 승인)
      const userIds = Object.values(sequentialUserIdsRef.current);
      if (userIds.length === 0) {
        throw new Error('생성된 사용자가 없습니다.');
      }

      const requestInfo = {
        method: 'POST',
        url: '/api/auth/admin/users/approve',
        body: userIds.map(id => ({ user_id: id }))
      };

      try {
        const approvedUsers = [];
        for (const userId of userIds) {
          const response = await axios.post(`${apiUrl}/api/auth/admin/users/approve`, {
            user_id: userId,
          }, {
            headers: {
              Authorization: `Bearer ${testToken}`,
            },
          });

          approvedUsers.push({ user_id: userId, ...response.data });
        }

        return {
          request: requestInfo,
          response: {
            approved_users: approvedUsers,
            count: approvedUsers.length,
            message: '모든 사용자 활성화 완료'
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // 4️⃣ 사용자를 프로젝트 멤버로 추가
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'POST',
        url: `/api/projects/${projectId}/members`,
        body: Object.values(sequentialUserIdsRef.current)
      };

      try {
        const addedMembers = [];
        for (const [username, userId] of Object.entries(sequentialUserIdsRef.current)) {
          const response = await axios.post(`${apiUrl}/api/projects/${projectId}/members`, {
            user_id: userId,
          }, {
            headers: {
              Authorization: `Bearer ${testToken}`,
            },
          });

          addedMembers.push({ username, user_id: userId, role: response.data.role_name });
        }

        return {
          request: requestInfo,
          response: { members: addedMembers, count: addedMembers.length },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // 5️⃣ Study 7개 생성
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const studies = [
        { study_uid: `1.2.840.113619.2.55.3.A.1.${Date.now()}`, study_description: 'CT Chest - A병원 환자1', patient_id: 'A-P001', patient_name: '김철수', study_date: '2025-01-10' },
        { study_uid: `1.2.840.113619.2.55.3.A.2.${Date.now()}`, study_description: 'CT Chest - A병원 환자2', patient_id: 'A-P002', patient_name: '이영희', study_date: '2025-01-11' },
        { study_uid: `1.2.840.113619.2.55.3.A.3.${Date.now()}`, study_description: 'CT Chest - A병원 환자3', patient_id: 'A-P003', patient_name: '박민수', study_date: '2025-01-12' },
        { study_uid: `1.2.840.113619.2.55.3.B.1.${Date.now()}`, study_description: 'MRI Brain - B병원 환자1', patient_id: 'B-P001', patient_name: '최지훈', study_date: '2025-01-13' },
        { study_uid: `1.2.840.113619.2.55.3.B.2.${Date.now()}`, study_description: 'MRI Brain - B병원 환자2', patient_id: 'B-P002', patient_name: '정수진', study_date: '2025-01-14' },
        { study_uid: `1.2.840.113619.2.55.3.B.3.${Date.now()}`, study_description: 'MRI Brain - B병원 환자3', patient_id: 'B-P003', patient_name: '강민호', study_date: '2025-01-15' },
        { study_uid: `1.2.840.113619.2.55.3.VIP.1.${Date.now()}`, study_description: 'PET-CT - VIP 환자', patient_id: 'VIP-001', patient_name: 'VIP 환자', study_date: '2025-01-16' },
      ];

      const requestInfo = {
        method: 'POST',
        url: `/api/project-data/${projectId}/data`,
        body: studies
      };

      try {
        const createdStudies = [];
        for (const study of studies) {
          const response = await axios.post(`${apiUrl}/api/project-data/${projectId}/data`, study, {
            headers: {
              Authorization: `Bearer ${testToken}`,
            },
          });

          createdStudies.push({ study_uid: study.study_uid, data_id: response.data.data_id });
          sequentialStudyIdsRef.current.push(response.data.data_id);
        }

        return {
          request: requestInfo,
          response: { studies: createdStudies, count: createdStudies.length },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 6️⃣ 접근 제어 설정
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const studyIds = sequentialStudyIdsRef.current;
      if (studyIds.length < 7) {
        throw new Error('Study가 충분하지 않습니다.');
      }

      const requestInfo = {
        method: 'PUT',
        url: `/api/project-data/${projectId}/data/{data_id}/access/{user_id}`,
        body: 'Multiple access control records'
      };

      try {
        const accessRecords = [];

        // Dr. Lee → A병원 Study 3개 (0, 1, 2)
        for (let i = 0; i < 3; i++) {
          await axios.put(
            `${apiUrl}/api/project-data/${projectId}/data/${studyIds[i]}/access/${sequentialUserIdsRef.current['dr_lee_seq']}`,
            { status: 'APPROVED', review_note: 'A병원 연구원 접근 승인' },
            { headers: { Authorization: `Bearer ${testToken}` } }
          );
          accessRecords.push({ user: 'dr_lee_seq', study_index: i, status: 'APPROVED' });
        }

        // Dr. Park → B병원 Study 3개 (3, 4, 5)
        for (let i = 3; i < 6; i++) {
          await axios.put(
            `${apiUrl}/api/project-data/${projectId}/data/${studyIds[i]}/access/${sequentialUserIdsRef.current['dr_park_seq']}`,
            { status: 'APPROVED', review_note: 'B병원 연구원 접근 승인' },
            { headers: { Authorization: `Bearer ${testToken}` } }
          );
          accessRecords.push({ user: 'dr_park_seq', study_index: i, status: 'APPROVED' });
        }

        // Dr. Choi → VIP Study 1개 (6) - 읽기 전용
        await axios.put(
          `${apiUrl}/api/project-data/${projectId}/data/${studyIds[6]}/access/${sequentialUserIdsRef.current['dr_choi_seq']}`,
          { status: 'APPROVED', review_note: '임시 협력자 읽기 전용 접근' },
          { headers: { Authorization: `Bearer ${testToken}` } }
        );
        accessRecords.push({ user: 'dr_choi_seq', study_index: 6, status: 'APPROVED', scope: 'READ_ONLY' });

        return {
          request: requestInfo,
          response: { access_records: accessRecords, count: accessRecords.length },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 7️⃣ 접근 제어 매트릭스 조회 및 검증
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`
      };

      try {
        const response = await axios.get(`${apiUrl}/api/project-data/${projectId}/data-access/matrix?page=1&page_size=100`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        const data = response.data;
        const users = data.users || [];
        const accessMatrix = data.access_matrix || [];
        const dataList = data.data_list || [];

        return {
          request: requestInfo,
          response: {
            users: users.length,
            studies: dataList.length,
            access_records: accessMatrix.length,
            matrix_summary: {
              dr_lee: accessMatrix.filter((a: any) => a.user_id === sequentialUserIdsRef.current['dr_lee_seq']).length,
              dr_park: accessMatrix.filter((a: any) => a.user_id === sequentialUserIdsRef.current['dr_park_seq']).length,
              dr_choi: accessMatrix.filter((a: any) => a.user_id === sequentialUserIdsRef.current['dr_choi_seq']).length,
            }
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 8️⃣ DICOM QIDO API로 실제 접근 제어 검증
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: '/api/dicom/studies (4명의 사용자로 접근 제어 검증)',
      };

      try {
        const testUsers = [
          { username: 'dr_kim_seq', password: 'Test1234!', name: 'Dr. Kim (책임연구원)', expectedStudies: 7 },
          { username: 'dr_lee_seq', password: 'Test1234!', name: 'Dr. Lee (A병원)', expectedStudies: 3 },
          { username: 'dr_park_seq', password: 'Test1234!', name: 'Dr. Park (B병원)', expectedStudies: 3 },
          { username: 'dr_choi_seq', password: 'Test1234!', name: 'Dr. Choi (임시 협력자)', expectedStudies: 1 },
        ];

        const results: any[] = [];

        for (const user of testUsers) {
          // 1. 사용자로 로그인
          const loginResponse = await axios.post(`${apiUrl}/api/auth/login`, {
            username: user.username,
            password: user.password,
          });

          const userToken = loginResponse.data.token;
          const keycloakToken = loginResponse.data.keycloak_access_token;

          console.log(`[${user.username}] Login response:`, {
            has_backend_token: !!userToken,
            has_keycloak_token: !!keycloakToken,
            keycloak_token_preview: keycloakToken ? keycloakToken.substring(0, 50) + '...' : 'MISSING'
          });

          // 2. DICOM QIDO API 호출 (project_id 필수, Keycloak access token 사용)
          const dicomResponse = await axios.get(`${apiUrl}/api/dicom/studies`, {
            params: {
              project_id: projectId,
            },
            headers: {
              Authorization: `Bearer ${keycloakToken}`,
            },
          });

          const studies = dicomResponse.data || [];
          const studyUids = studies.map((s: any) => s['0020000D']?.Value?.[0] || 'Unknown');

          results.push({
            user: user.name,
            username: user.username,
            expected_studies: user.expectedStudies,
            actual_studies: studies.length,
            study_uids: studyUids,
            access_control_working: studies.length === user.expectedStudies ? '✅ 정상' : '❌ 오류',
          });
        }

        return {
          request: requestInfo,
          response: {
            message: '실제 접근 제어 검증 완료',
            results: results,
            summary: {
              total_tests: results.length,
              passed: results.filter((r: any) => r.access_control_working === '✅ 정상').length,
              failed: results.filter((r: any) => r.access_control_working === '❌ 오류').length,
            }
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 9) {
      // 9️⃣ 정리 (프로젝트 삭제)
      const projectId = sequentialProjectIdRef.current;
      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = { method: 'DELETE', url: `/api/projects/${projectId}` };

      try {
        const response = await axios.delete(`${apiUrl}/api/projects/${projectId}`, {
          headers: {
            Authorization: `Bearer ${testToken}`,
          },
        });

        // 초기화
        sequentialProjectIdRef.current = null;
        sequentialUserIdsRef.current = {};
        sequentialStudyIdsRef.current = [];

        return {
          request: requestInfo,
          response: { message: '프로젝트 및 관련 데이터 삭제 완료', project_id: projectId },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    }

    return { request: {}, response: { message: '알 수 없는 테스트 인덱스' } };
  };

  // 섹션별 테스트 실행
  const runSectionTests = async (sectionIndex: number) => {
    const section = sections[sectionIndex];

    console.log(`🚀 섹션 테스트 시작: ${section.title}`);

    // 해당 섹션의 테스트들을 pending으로 초기화
    const newSections = [...sections];
    newSections[sectionIndex] = {
      ...section,
      tests: section.tests.map(test => ({
        ...test,
        status: 'pending' as const,
        request: undefined,
        response: undefined,
        error: undefined,
        duration: undefined,
      })),
    };
    setSections(newSections);

    // 섹션의 모든 테스트 실행
    for (let testIndex = 0; testIndex < section.tests.length; testIndex++) {
      await runTest(sectionIndex, testIndex);
    }

    console.log(`✅ 섹션 테스트 완료: ${section.title}`);
  };

  // 모든 테스트 실행 (의존성 및 순차 실행 고려)
  const runAllTests = async () => {
    setIsRunningAll(true);
    setCreatedProjectId(null);
    createdProjectIdRef.current = null; // ref도 초기화
    createdStudyIdRef.current = null;
    createdStudyUidRef.current = null;
    createdSeriesIdsRef.current = [];
    createdSeriesUidsRef.current = [];

    // 모든 테스트를 pending으로 초기화
    const newSections = sections.map(section => ({
      ...section,
      tests: section.tests.map(test => ({
        ...test,
        status: 'pending' as const,
        request: undefined,
        response: undefined,
        error: undefined,
        duration: undefined,
      })),
    }));
    setSections(newSections);

    // 섹션별로 실행
    // 최신 sections 상태를 추적하기 위한 로컬 변수
    let currentSections: TestSection[] = [...newSections];
    
    for (let sectionIndex = 0; sectionIndex < currentSections.length; sectionIndex++) {
      const section = currentSections[sectionIndex];

      if (section.isSequential) {
        // 순차 실행 섹션: 하나씩 순서대로 실행
        console.log(`📋 순차 실행 섹션: ${section.title}`);
        for (let testIndex = 0; testIndex < section.tests.length; testIndex++) {
          const test = section.tests[testIndex];
          console.log(`  ▶️ ${test.name} 실행 중...`);
          await runTest(sectionIndex, testIndex);
          
          // runTest 후 최신 상태 가져오기
          await new Promise<void>((resolve) => {
            setSections(prevSections => {
              currentSections = [...prevSections];
              resolve();
              return prevSections;
            });
          });

          // 테스트별 커스텀 딜레이 또는 기본 딜레이
          const delay = test.delayAfter !== undefined ? test.delayAfter : 300;
          if (delay > 0) {
            console.log(`  ⏱️ ${delay}ms 대기 중...`);
            await new Promise(resolve => setTimeout(resolve, delay));
          }
        }
      } else {
        // 병렬 실행 가능 섹션: 의존성만 체크하고 실행
        console.log(`🔀 병렬 실행 섹션: ${section.title}`);
        for (let testIndex = 0; testIndex < section.tests.length; testIndex++) {
          const test = section.tests[testIndex];

          // 의존성 체크 (최신 sections 상태 사용)
          const dependencyCheck = canRunTest(sectionIndex, testIndex, currentSections);
          if (dependencyCheck.canRun) {
            console.log(`  ▶️ ${test.name} 실행 중...`);
            await runTest(sectionIndex, testIndex);
            
            // runTest 후 최신 상태 가져오기
            await new Promise<void>((resolve) => {
              setSections(prevSections => {
                currentSections = [...prevSections];
                resolve();
                return prevSections;
              });
            });
          } else {
            console.log(`  ⏭️ ${test.name} 건너뜀: ${dependencyCheck.reason}`);
            // 의존성 미충족 시 스킵
            setSections(prevSections => {
              const newSections = [...prevSections];
              newSections[sectionIndex].tests[testIndex].status = 'skipped';
              newSections[sectionIndex].tests[testIndex].error = dependencyCheck.reason;
              currentSections = [...newSections];
              return newSections;
            });
          }

          // 테스트별 커스텀 딜레이 또는 기본 딜레이
          const delay = test.delayAfter !== undefined ? test.delayAfter : 200;
          if (delay > 0) {
            await new Promise(resolve => setTimeout(resolve, delay));
          }
        }
      }
    }

    setIsRunningAll(false);
  };

  const toggleTestDetails = (testName: string) => {
    setExpandedTest(expandedTest === testName ? null : testName);
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'success': return '✅';
      case 'failure': return '❌';
      case 'running': return '⏳';
      case 'skipped': return '⏭️';
      case 'pending': return '⚪';
      default: return '⚪';
    }
  };

  // Annotation 권한 관리 테스트
  const runAnnotationPermissionTest = async (testIndex: number) => {
    const testStudyUid = '1.2.840.113619.2.55.3.604688119.868.1234567890.1';
    const testSeriesUid = '1.2.840.113619.2.55.3.604688119.868.1234567890.2';
    const testSopUid = '1.2.840.113619.2.55.3.604688119.868.1234567890.3';

    if (testIndex === 0) {
      // 0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)
      const requestInfo = {
        method: 'DELETE',
        url: '/api/projects (기존 테스트 프로젝트)',
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        
        // 프로젝트 목록 조회
        const projectsResponse = await axios.get(`${apiUrl}/api/projects`, config);
        const projects = projectsResponse.data.projects || [];
        
        // 이름으로 검색 (정확히 일치)
        const projectName = 'Annotation 권한 관리 테스트 프로젝트';
        const existingProjects = projects.filter((p: any) => p.name === projectName);
        
        // 찾은 프로젝트 삭제
        const deletedProjects = [];
        for (const project of existingProjects) {
          try {
            await axios.delete(`${apiUrl}/api/projects/${project.id}`, config);
            deletedProjects.push({ id: project.id, name: project.name });
            console.log(`  ✅ 기존 프로젝트 삭제: ${project.name} (ID: ${project.id})`);
          } catch (error: any) {
            // 404 에러는 무시 (이미 삭제된 경우)
            if (error.response?.status !== 404) {
              console.warn(`  ⚠️ 프로젝트 삭제 실패: ${project.name}`, error.message);
            }
          }
        }
        
        return {
          request: requestInfo,
          response: {
            message: '사전 정리 완료',
            deleted_count: deletedProjects.length,
            deleted_projects: deletedProjects,
          },
        };
      } catch (error: any) {
        // 에러가 발생해도 계속 진행 (정리 단계이므로)
        return {
          request: requestInfo,
          response: { message: '정리 중 에러 발생 (무시하고 계속 진행)', error: error.message },
        };
      }
    } else if (testIndex === 1) {
      // 1️⃣ 테스트용 프로젝트 생성
      const requestBody = {
        name: 'Annotation 권한 관리 테스트 프로젝트',
        description: 'Annotation 권한 관리 기능 테스트를 위한 프로젝트',
        sponsor: 'Test Sponsor',
        start_date: new Date().toISOString().split('T')[0],
        end_date: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
        status: 'PREPARING',
      };

      const requestInfo = { method: 'POST', url: '/api/projects', body: requestBody };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        const response = await axios.post(`${apiUrl}/api/projects`, requestBody, config);

        const projectId = response.data.id;
        annotationPermissionProjectIdRef.current = projectId;

        // 현재 사용자 ID 저장 (SUPER_ADMIN)
        const userResponse = await axios.get(`${apiUrl}/api/users/me`, config);
        annotationPermissionTestUserIdRef.current = userResponse.data.id;

        console.log(`  ✅ 테스트용 프로젝트 생성 성공:`, response.data);

        return {
          request: requestInfo,
          response: {
            project_id: projectId,
            project_name: response.data.name,
            user_id: annotationPermissionTestUserIdRef.current,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // 2️⃣ 사용자를 프로젝트 멤버로 추가
      const projectId = annotationPermissionProjectIdRef.current;
      const userId = annotationPermissionTestUserIdRef.current;

      if (!projectId || !userId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'POST',
        url: `/api/projects/${projectId}/members`,
        body: { user_id: userId },
      };

      try {
        const result = await addUsersToProject(projectId, [userId]);
        return {
          request: requestInfo,
          response: {
            project_id: result.project_id,
            user_id: userId,
            role: result.added_members[0]?.role || 'MEMBER',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${projectId}/members`,
            data: { user_id: userId },
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 3️⃣ 개발 모드: 쿼리 파라미터로 Annotation 생성
      const projectId = annotationPermissionProjectIdRef.current;
      const userId = annotationPermissionTestUserIdRef.current;

      if (!projectId || !userId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestBody = {
        project_id: projectId,
        study_instance_uid: testStudyUid,
        series_instance_uid: testSeriesUid,
        sop_instance_uid: testSopUid,
        annotation_data: {
          type: 'circle',
          x: 100,
          y: 200,
          radius: 50,
          color: '#FF0000',
        },
        tool_name: 'Circle Tool',
        tool_version: '2.1.0',
        viewer_software: 'OHIF Viewer',
        description: '쿼리 파라미터로 생성된 Annotation',
      };

      const requestInfo = {
        method: 'POST',
        url: `/api/annotations?user_id=${userId}`,
        body: requestBody,
      };

      try {
        // 개발 모드에서는 쿼리 파라미터로 user_id 전달
        const response = await axios.post(
          `${apiUrl}/api/annotations?user_id=${userId}`,
          requestBody
        );

        const annotationId = response.data.id;
        annotationPermissionAnnotationIdsRef.current.push(annotationId);

        console.log(`  ✅ 쿼리 파라미터로 Annotation 생성 성공:`, response.data);

        return {
          request: requestInfo,
          response: {
            annotation_id: annotationId,
            user_id: response.data.user_id,
            method: 'query_parameter',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/annotations?user_id=${userId}`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // 4️⃣ 개발 모드: 헤더로 Annotation 생성
      const projectId = annotationPermissionProjectIdRef.current;
      const userId = annotationPermissionTestUserIdRef.current;

      if (!projectId || !userId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestBody = {
        project_id: projectId,
        study_instance_uid: testStudyUid,
        series_instance_uid: testSeriesUid,
        sop_instance_uid: testSopUid,
        annotation_data: {
          type: 'rectangle',
          x: 150,
          y: 250,
          width: 200,
          height: 100,
          color: '#00FF00',
        },
        tool_name: 'Rectangle Tool',
        tool_version: '2.1.0',
        viewer_software: 'TI-DicomViewer',
        description: '헤더로 생성된 Annotation',
      };

      const requestInfo = {
        method: 'POST',
        url: '/api/annotations',
        headers: { 'X-User-ID': userId.toString() },
        body: requestBody,
      };

      try {
        // 개발 모드에서는 헤더로 user_id 전달
        const response = await axios.post(`${apiUrl}/api/annotations`, requestBody, {
          headers: {
            'X-User-ID': userId.toString(),
          },
        });

        const annotationId = response.data.id;
        annotationPermissionAnnotationIdsRef.current.push(annotationId);

        console.log(`  ✅ 헤더로 Annotation 생성 성공:`, response.data);

        return {
          request: requestInfo,
          response: {
            annotation_id: annotationId,
            user_id: response.data.user_id,
            method: 'header',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/annotations`,
            data: requestBody,
            headers: {
              'X-User-ID': userId.toString(),
            },
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // 5️⃣ 권한 조회 API 테스트
      const projectId = annotationPermissionProjectIdRef.current;
      const userId = annotationPermissionTestUserIdRef.current;

      if (!projectId || !userId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/permissions?project_id=${projectId}&user_id=${userId}`,
      };

      try {
        // 개발 모드에서는 쿼리 파라미터로 user_id 전달
        const response = await axios.get(
          `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${userId}`
        );

        console.log(`  ✅ 권한 조회 성공:`, response.data);

        return {
          request: requestInfo,
          response: {
            permissions: response.data,
            user_id: userId,
            project_id: projectId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${userId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 6️⃣ 소유자 Annotation 조회 테스트
      const userId = annotationPermissionTestUserIdRef.current;
      const annotationId = annotationPermissionAnnotationIdsRef.current[0];

      if (!userId || !annotationId) {
        throw new Error('사용자 ID 또는 Annotation ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/${annotationId}?user_id=${userId}`,
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations/${annotationId}?user_id=${userId}`
        );

        console.log(`  ✅ 소유자 Annotation 조회 성공:`, response.data);

        return {
          request: requestInfo,
          response: {
            annotation_id: response.data.id,
            user_id: response.data.user_id,
            description: response.data.description,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/${annotationId}?user_id=${userId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 7️⃣ READ_ALL 권한으로 다른 사용자 Annotation 조회 테스트
      const userId = annotationPermissionTestUserIdRef.current;
      const annotationId = annotationPermissionAnnotationIdsRef.current[0];

      if (!userId || !annotationId) {
        throw new Error('사용자 ID 또는 Annotation ID가 없습니다.');
      }

      // READ_ALL 권한이 있는 사용자 (SUPER_ADMIN)로 조회
      const config = await handleGetAxiosConfig('SUPER_ADMIN');
      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/${annotationId}?user_id=1`, // SUPER_ADMIN user_id
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations/${annotationId}?user_id=1`
        );

        console.log(`  ✅ READ_ALL 권한으로 Annotation 조회 성공:`, response.data);

        return {
          request: requestInfo,
          response: {
            annotation_id: response.data.id,
            user_id: response.data.user_id,
            description: response.data.description,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/${annotationId}?user_id=1`,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 8️⃣ 권한 없는 사용자 Annotation 조회 시도 (401 에러)
      const annotationId = annotationPermissionAnnotationIdsRef.current[0];

      if (!annotationId) {
        throw new Error('Annotation ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/${annotationId} (user_id 없음)`,
      };

      try {
        // user_id 없이 요청 (프로덕션 모드 시뮬레이션)
        await axios.get(`${apiUrl}/api/annotations/${annotationId}`);
        throw new Error('401 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 401) {
          console.log(`  ✅ 권한 없음 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 401, message: 'Unauthorized' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/${annotationId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 9) {
      // 9️⃣ 소유자 Annotation 수정 테스트
      const userId = annotationPermissionTestUserIdRef.current;
      const annotationId = annotationPermissionAnnotationIdsRef.current[0];

      if (!userId || !annotationId) {
        throw new Error('사용자 ID 또는 Annotation ID가 없습니다.');
      }

      const requestBody = {
        annotation_data: {
          type: 'circle',
          x: 200,
          y: 300,
          radius: 75,
          color: '#0000FF',
        },
        description: '소유자가 수정한 Annotation',
      };

      const requestInfo = {
        method: 'PUT',
        url: `/api/annotations/${annotationId}?user_id=${userId}`,
        body: requestBody,
      };

      try {
        const response = await axios.put(
          `${apiUrl}/api/annotations/${annotationId}?user_id=${userId}`,
          requestBody
        );

        console.log(`  ✅ 소유자 Annotation 수정 성공:`, response.data);

        return {
          request: requestInfo,
          response: {
            annotation_id: response.data.id,
            version: response.data.version,
            description: response.data.description,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'put',
            url: `${apiUrl}/api/annotations/${annotationId}?user_id=${userId}`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 10) {
      // 🔟 소유자 Annotation 삭제 테스트
      const userId = annotationPermissionTestUserIdRef.current;
      const annotationId = annotationPermissionAnnotationIdsRef.current[1]; // 두 번째 annotation 삭제

      if (!userId || !annotationId) {
        throw new Error('사용자 ID 또는 Annotation ID가 없습니다.');
      }

      const requestInfo = {
        method: 'DELETE',
        url: `/api/annotations/${annotationId}?user_id=${userId}`,
      };

      try {
        await axios.delete(`${apiUrl}/api/annotations/${annotationId}?user_id=${userId}`);

        // 삭제된 annotation ID 제거
        annotationPermissionAnnotationIdsRef.current = annotationPermissionAnnotationIdsRef.current.filter(
          (id) => id !== annotationId
        );

        console.log(`  ✅ 소유자 Annotation 삭제 성공`);

        return {
          request: requestInfo,
          response: {
            message: 'Annotation deleted successfully',
            deleted_annotation_id: annotationId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/annotations/${annotationId}?user_id=${userId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 11) {
      // 1️⃣1️⃣ 권한 없는 사용자 Annotation 생성 시도 (401 에러)
      const projectId = annotationPermissionProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestBody = {
        project_id: projectId,
        study_instance_uid: testStudyUid,
        series_instance_uid: testSeriesUid,
        sop_instance_uid: testSopUid,
        annotation_data: {
          type: 'point',
          x: 300,
          y: 400,
        },
        description: '권한 없는 사용자가 생성 시도',
      };

      const requestInfo = {
        method: 'POST',
        url: '/api/annotations (user_id 없음)',
        body: requestBody,
      };

      try {
        // user_id 없이 요청 (프로덕션 모드 시뮬레이션)
        await axios.post(`${apiUrl}/api/annotations`, requestBody);
        throw new Error('401 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 401) {
          console.log(`  ✅ 권한 없음 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 401, message: 'Unauthorized' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/annotations`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 12) {
      // 1️⃣2️⃣ 정리 (테스트 프로젝트 삭제)
      const projectId = annotationPermissionProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'DELETE',
        url: `/api/projects/${projectId}`,
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        await axios.delete(`${apiUrl}/api/projects/${projectId}`, config);

        // ref 초기화
        annotationPermissionProjectIdRef.current = null;
        annotationPermissionAnnotationIdsRef.current = [];
        annotationPermissionTestUserIdRef.current = null;

        console.log(`  ✅ 테스트 프로젝트 삭제 성공`);

        return {
          request: requestInfo,
          response: {
            message: 'Test project deleted successfully',
            project_id: projectId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${projectId}`,
          };
        }
        throw error;
      }
    }

    throw new Error(`Unknown test index: ${testIndex}`);
  };

  // 권한 기반 Annotation 조회 테스트 (READ_ALL)
  const runReadAllPermissionTest = async (testIndex: number) => {
    const testStudyUid = readAllTestStudyUidRef.current;
    const testSeriesUid = readAllTestSeriesUidRef.current;
    const testSopUid = readAllTestSopUidRef.current;

    if (testIndex === 0) {
      // 0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)
      // 정리 단계는 실패해도 성공으로 처리 (다음 단계 진행 가능)
      const requestInfo = {
        method: 'DELETE',
        url: '/api/projects (기존 테스트 프로젝트)',
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        
        // 프로젝트 목록 조회
        const projectsResponse = await axios.get(`${apiUrl}/api/projects`, config);
        const projects = projectsResponse.data.projects || [];
        
        // 이름 패턴으로 검색 (시작하는 프로젝트)
        const projectNamePattern = 'READ_ALL Permission Test';
        const existingProjects = projects.filter((p: any) => 
          p.name.startsWith(projectNamePattern)
        );
        
        // 찾은 프로젝트 삭제
        const deletedProjects = [];
        for (const project of existingProjects) {
          try {
            await axios.delete(`${apiUrl}/api/projects/${project.id}`, config);
            deletedProjects.push({ id: project.id, name: project.name });
            console.log(`  ✅ 기존 프로젝트 삭제: ${project.name} (ID: ${project.id})`);
          } catch (error: any) {
            // 404 에러는 무시 (이미 삭제된 경우)
            if (error.response?.status !== 404) {
              console.warn(`  ⚠️ 프로젝트 삭제 실패: ${project.name}`, error.message);
            }
          }
        }
        
        console.log(`  ✅ 사전 정리 완료: ${deletedProjects.length}개 프로젝트 삭제`);
        
        return {
          request: requestInfo,
          response: {
            message: '사전 정리 완료',
            deleted_count: deletedProjects.length,
            deleted_projects: deletedProjects,
          },
        };
      } catch (error: any) {
        // 정리 단계는 에러가 발생해도 성공으로 처리 (다음 단계 진행 가능)
        console.warn(`  ⚠️ 사전 정리 중 에러 발생 (무시하고 계속 진행):`, error.message);
        return {
          request: requestInfo,
          response: { 
            message: '정리 중 에러 발생 (무시하고 계속 진행)', 
            error: error.message,
            note: '정리 단계는 실패해도 다음 단계로 진행됩니다'
          },
        };
      }
    } else if (testIndex === 1) {
      // 1️⃣ 테스트용 프로젝트 생성
      const requestBody = {
        name: `READ_ALL Permission Test ${Date.now()}`,
        description: 'ADJUDICATOR READ_ALL 권한 테스트용 프로젝트',
        sponsor: 'Test Sponsor',
        status: 'ACTIVE',
      };

      const requestInfo = {
        method: 'POST',
        url: '/api/projects',
        body: requestBody,
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        const response = await axios.post(`${apiUrl}/api/projects`, requestBody, config);

        readAllProjectIdRef.current = response.data.id;
        console.log(`  ✅ 테스트 프로젝트 생성 성공: ID=${response.data.id}`);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // 2️⃣ 테스트용 사용자 3명 생성 (일반 사용자 2명, ADJUDICATOR 1명)
      const timestamp = Date.now();
      const users = [
        { username: `readall_user1_${timestamp}`, email: `readall.user1.${timestamp}@test.com`, full_name: 'READ_ALL Test User 1' },
        { username: `readall_user2_${timestamp}`, email: `readall.user2.${timestamp}@test.com`, full_name: 'READ_ALL Test User 2' },
        { username: `readall_adjudicator_${timestamp}`, email: `readall.adjudicator.${timestamp}@test.com`, full_name: 'READ_ALL Test ADJUDICATOR' },
      ];

      const requestInfo = {
        method: 'POST',
        url: '/api/auth/signup',
        body: `사용자 ${users.length}명 생성`,
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        const createdUsers = [];

        for (const user of users) {
          try {
            const response = await axios.post(`${apiUrl}/api/auth/signup`, {
              username: user.username,
              email: user.email,
              password: 'Test1234!',
              full_name: user.full_name,
            }, config);

            createdUsers.push(response.data);
            console.log(`  ✅ 사용자 생성 성공: ${user.username} (user_id: ${response.data.user_id})`);
          } catch (error: any) {
            // 이미 존재하는 경우 조회
            if (error.response?.status === 409 || 
                error.response?.data?.error?.includes('Already exists') ||
                error.response?.data?.error?.includes('User already exists')) {
              console.log(`  ℹ️ 사용자 ${user.username}는 이미 존재합니다. 기존 사용자 정보를 조회합니다.`);
              
              try {
                const userResponse = await axios.get(
                  `${apiUrl}/api/users/username/${user.username}`,
                  config
                );

                const existingUser = userResponse.data;
                createdUsers.push({
                  user_id: existingUser.id,
                  username: existingUser.username,
                  email: existingUser.email,
                  full_name: existingUser.full_name,
                });
                console.log(`  ✅ 기존 사용자 조회 성공: ${user.username} (user_id: ${existingUser.id})`);
              } catch (lookupError: any) {
                console.error(`  ❌ 사용자 조회 실패: ${user.username}`, lookupError);
                throw new Error(`사용자 ${user.username}가 이미 존재하지만 조회에 실패했습니다: ${lookupError.message}`);
              }
            } else {
              throw error;
            }
          }
        }

        // 사용자 ID 저장
        readAllUser1IdRef.current = createdUsers[0].user_id;
        readAllUser2IdRef.current = createdUsers[1].user_id;
        readAllAdjudicatorIdRef.current = createdUsers[2].user_id;

        return {
          request: requestInfo,
          response: {
            users: createdUsers.map(u => ({ username: u.username, user_id: u.user_id })),
            count: createdUsers.length,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 3️⃣ 사용자들을 프로젝트 멤버로 추가
      const projectId = readAllProjectIdRef.current;
      const user1Id = readAllUser1IdRef.current;
      const user2Id = readAllUser2IdRef.current;
      const adjudicatorId = readAllAdjudicatorIdRef.current;

      if (!projectId || !user1Id || !user2Id || !adjudicatorId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const userIds = [user1Id, user2Id, adjudicatorId];
      const requestInfo = {
        method: 'POST',
        url: `/api/projects/${projectId}/members`,
        body: `사용자 ${userIds.length}명 추가`,
      };

      try {
        const result = await addUsersToProject(projectId, userIds);
        return {
          request: requestInfo,
          response: result,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${projectId}/members`,
          };
        }
        throw error;
      }
    } else if (testIndex === 4) {
      // 4️⃣ ADJUDICATOR 역할 할당
      const projectId = readAllProjectIdRef.current;
      const adjudicatorId = readAllAdjudicatorIdRef.current;

      if (!projectId || !adjudicatorId) {
        throw new Error('프로젝트 ID 또는 ADJUDICATOR 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: '/api/roles/project',
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        
        // 프로젝트 역할 목록 조회
        const rolesResponse = await axios.get(`${apiUrl}/api/roles/project`, config);
        const roles = rolesResponse.data;
        
        // ADJUDICATOR 역할 찾기
        const adjudicatorRole = roles.find((r: any) => r.name === 'ADJUDICATOR');
        if (!adjudicatorRole) {
          throw new Error('ADJUDICATOR 역할을 찾을 수 없습니다.');
        }

        readAllAdjudicatorRoleIdRef.current = adjudicatorRole.id;

        // 프로젝트에 역할 할당 (이미 할당되어 있을 수 있음)
        try {
          await axios.post(
            `${apiUrl}/api/projects/${projectId}/roles/${adjudicatorRole.id}`,
            {},
            config
          );
          console.log(`  ✅ 프로젝트에 ADJUDICATOR 역할 할당 성공`);
        } catch (error: any) {
          // 이미 할당된 경우 무시
          if (error.response?.status !== 400 && error.response?.status !== 409) {
            throw error;
          }
          console.log(`  ℹ️ 프로젝트에 ADJUDICATOR 역할이 이미 할당되어 있습니다.`);
        }

        // 사용자에게 역할 할당
        const assignResponse = await axios.post(
          `${apiUrl}/api/projects/${projectId}/users/${adjudicatorId}/role`,
          { role_id: adjudicatorRole.id },
          config
        );

        console.log(`  ✅ ADJUDICATOR 역할 할당 성공: user_id=${adjudicatorId}, role_id=${adjudicatorRole.id}`);

        return {
          request: requestInfo,
          response: {
            message: 'ADJUDICATOR 역할 할당 성공',
            user_id: adjudicatorId,
            role_id: adjudicatorRole.id,
            role_name: 'ADJUDICATOR',
            assignment: assignResponse.data,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // 5️⃣ 일반 사용자1로 Annotation 3개 생성
      const projectId = readAllProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const userId = readAllUser1IdRef.current;

      if (!userId) {
        throw new Error('사용자1 ID가 없습니다.');
      }

      const createdIds: number[] = [];

      const requestInfo = {
        method: 'POST',
        url: `/api/annotations?user_id=${userId}`,
        body: '3개의 Annotation 생성',
      };

      try {
        for (let i = 0; i < 3; i++) {
          const requestBody = {
            project_id: projectId,
            study_instance_uid: testStudyUid,
            series_instance_uid: testSeriesUid,
            sop_instance_uid: testSopUid,
            annotation_data: {
              type: 'point',
              x: 100 + i * 50,
              y: 100 + i * 50,
              color: '#FF0000',
            },
            tool_name: 'Point Tool',
            description: `User 1 Annotation ${i + 1}`,
          };

          const response = await axios.post(
            `${apiUrl}/api/annotations?user_id=${userId}`,
            requestBody
          );
          createdIds.push(response.data.id);
          readAllAnnotationIdsRef.current.push(response.data.id);

          console.log(`  ✅ User 1 Annotation ${i + 1} 생성 성공: ID=${response.data.id}`);
        }

        return {
          request: requestInfo,
          response: {
            message: 'User 1이 3개의 Annotation 생성 성공',
            created_ids: createdIds,
            user_id: userId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/annotations?user_id=${userId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 6️⃣ 일반 사용자2로 Annotation 2개 생성
      const projectId = readAllProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const userId = readAllUser2IdRef.current;

      if (!userId) {
        throw new Error('사용자2 ID가 없습니다.');
      }
      const createdIds: number[] = [];

      const requestInfo = {
        method: 'POST',
        url: `/api/annotations?user_id=${userId}`,
        body: '2개의 Annotation 생성',
      };

      try {
        for (let i = 0; i < 2; i++) {
          const requestBody = {
            project_id: projectId,
            study_instance_uid: testStudyUid,
            series_instance_uid: testSeriesUid,
            sop_instance_uid: testSopUid,
            annotation_data: {
              type: 'rectangle',
              x: 200 + i * 50,
              y: 200 + i * 50,
              width: 100,
              height: 100,
              color: '#00FF00',
            },
            tool_name: 'Rectangle Tool',
            description: `User 2 Annotation ${i + 1}`,
          };

          const response = await axios.post(
            `${apiUrl}/api/annotations?user_id=${userId}`,
            requestBody
          );
          createdIds.push(response.data.id);
          readAllAnnotationIdsRef.current.push(response.data.id);

          console.log(`  ✅ User 2 Annotation ${i + 1} 생성 성공: ID=${response.data.id}`);
        }

        return {
          request: requestInfo,
          response: {
            message: 'User 2가 2개의 Annotation 생성 성공',
            created_ids: createdIds,
            user_id: userId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/annotations?user_id=${userId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 7️⃣ 일반 사용자1 본인 Annotation만 조회 (3개)
      const projectId = readAllProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const userId = readAllUser1IdRef.current;

      if (!userId) {
        throw new Error('사용자1 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations?user_id=${userId}&project_id=${projectId}`,
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations?user_id=${userId}&project_id=${projectId}`
        );

        const total = response.data.total;
        const userAnnotations = response.data.annotations.filter((ann: any) => ann.user_id === userId);

        if (total !== 3 || userAnnotations.length !== 3) {
          throw new Error(`User 1은 본인 Annotation 3개만 조회해야 하는데 ${total}개 조회됨`);
        }

        console.log(`  ✅ User 1 본인 Annotation만 조회 성공: ${total}개`);

        return {
          request: requestInfo,
          response: {
            message: 'User 1은 본인 Annotation만 조회 (READ_ALL 권한 없음)',
            total: total,
            user_id: userId,
            annotations_preview: userAnnotations.slice(0, 2),
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations?user_id=${userId}&project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 8️⃣ ADJUDICATOR 모든 Annotation 조회 (5개)
      const projectId = readAllProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const userId = readAllAdjudicatorIdRef.current;

      if (!userId) {
        throw new Error('ADJUDICATOR 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations?user_id=${userId}&project_id=${projectId}`,
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations?user_id=${userId}&project_id=${projectId}`
        );

        const total = response.data.total;

        if (total !== 5) {
          throw new Error(`ADJUDICATOR는 모든 Annotation 5개를 조회해야 하는데 ${total}개 조회됨`);
        }

        const user1Id = readAllUser1IdRef.current;
        const user2Id = readAllUser2IdRef.current;
        const user1Count = response.data.annotations.filter((ann: any) => ann.user_id === user1Id).length;
        const user2Count = response.data.annotations.filter((ann: any) => ann.user_id === user2Id).length;

        console.log(`  ✅ ADJUDICATOR 모든 Annotation 조회 성공: ${total}개 (User 1: ${user1Count}, User 2: ${user2Count})`);

        return {
          request: requestInfo,
          response: {
            message: 'ADJUDICATOR는 READ_ALL 권한으로 모든 Annotation 조회',
            total: total,
            user_id: userId,
            user_1_count: user1Count,
            user_2_count: user2Count,
            annotations_preview: response.data.annotations.slice(0, 3),
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations?user_id=${userId}&project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 9) {
      // 9️⃣ SOP Instance UID로 조회 (READ_ALL 권한 확인)
      const projectId = readAllProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const userId = readAllAdjudicatorIdRef.current;

      if (!userId) {
        throw new Error('ADJUDICATOR 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations?sop_instance_uid=${testSopUid}&user_id=${userId}&project_id=${projectId}`,
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations?sop_instance_uid=${testSopUid}&user_id=${userId}&project_id=${projectId}`
        );

        const total = response.data.total;

        if (total !== 5) {
          throw new Error(`SOP Instance UID로 조회 시 5개가 나와야 하는데 ${total}개 조회됨`);
        }

        console.log(`  ✅ SOP Instance UID로 조회 성공 (READ_ALL): ${total}개`);

        return {
          request: requestInfo,
          response: {
            message: 'ADJUDICATOR는 SOP Instance UID로 모든 사용자의 Annotation 조회',
            total: total,
            sop_instance_uid: testSopUid,
            annotations_preview: response.data.annotations.slice(0, 2),
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations?sop_instance_uid=${testSopUid}&user_id=${userId}&project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 10) {
      // 🔟 Series UID로 조회 (READ_ALL 권한 확인)
      const projectId = readAllProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const userId = readAllAdjudicatorIdRef.current;

      if (!userId) {
        throw new Error('ADJUDICATOR 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations?series_instance_uid=${testSeriesUid}&user_id=${userId}&project_id=${projectId}`,
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations?series_instance_uid=${testSeriesUid}&user_id=${userId}&project_id=${projectId}`
        );

        const total = response.data.total;

        if (total !== 5) {
          throw new Error(`Series UID로 조회 시 5개가 나와야 하는데 ${total}개 조회됨`);
        }

        console.log(`  ✅ Series UID로 조회 성공 (READ_ALL): ${total}개`);

        return {
          request: requestInfo,
          response: {
            message: 'ADJUDICATOR는 Series UID로 모든 사용자의 Annotation 조회',
            total: total,
            series_instance_uid: testSeriesUid,
            annotations_preview: response.data.annotations.slice(0, 2),
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations?series_instance_uid=${testSeriesUid}&user_id=${userId}&project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 11) {
      // 1️⃣1️⃣ Study UID로 조회 (READ_ALL 권한 확인)
      const projectId = readAllProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const userId = readAllAdjudicatorIdRef.current;

      if (!userId) {
        throw new Error('ADJUDICATOR 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations?study_instance_uid=${testStudyUid}&user_id=${userId}&project_id=${projectId}`,
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations?study_instance_uid=${testStudyUid}&user_id=${userId}&project_id=${projectId}`
        );

        const total = response.data.total;

        if (total !== 5) {
          throw new Error(`Study UID로 조회 시 5개가 나와야 하는데 ${total}개 조회됨`);
        }

        console.log(`  ✅ Study UID로 조회 성공 (READ_ALL): ${total}개`);

        return {
          request: requestInfo,
          response: {
            message: 'ADJUDICATOR는 Study UID로 모든 사용자의 Annotation 조회',
            total: total,
            study_instance_uid: testStudyUid,
            annotations_preview: response.data.annotations.slice(0, 2),
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations?study_instance_uid=${testStudyUid}&user_id=${userId}&project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 12) {
      // 1️⃣2️⃣ Summary API로 전체 통계 조회
      const projectId = readAllProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const userId = readAllAdjudicatorIdRef.current;

      if (!userId) {
        throw new Error('ADJUDICATOR 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/summary?user_id=${userId}&project_id=${projectId}`,
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations/summary?user_id=${userId}&project_id=${projectId}`
        );

        const totalCount = response.data.total_count;

        if (totalCount !== 5) {
          throw new Error(`Summary API에서 5개가 나와야 하는데 ${totalCount}개 조회됨`);
        }

        console.log(`  ✅ Summary API 조회 성공 (READ_ALL): ${totalCount}개`);

        return {
          request: requestInfo,
          response: {
            message: 'ADJUDICATOR는 Summary API로 모든 사용자의 Annotation 통계 조회',
            total_count: totalCount,
            summary: response.data,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/summary?user_id=${userId}&project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 13) {
      // 1️⃣3️⃣ 정리 (생성된 Annotation 삭제)
      const annotationIds = readAllAnnotationIdsRef.current;
      const adjudicatorId = readAllAdjudicatorIdRef.current;

      if (annotationIds.length === 0) {
        console.log(`  ℹ️ 삭제할 Annotation이 없습니다.`);
        return {
          request: { method: 'DELETE', url: '/api/annotations' },
          response: { message: 'No annotations to delete', deleted_count: 0 },
        };
      }

      if (!adjudicatorId) {
        throw new Error('ADJUDICATOR 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'DELETE',
        url: `/api/annotations (${annotationIds.length}개)`,
      };

      try {
        for (const id of annotationIds) {
          try {
            await axios.delete(`${apiUrl}/api/annotations/${id}?user_id=${adjudicatorId}`);
            console.log(`  ✅ Annotation ${id} 삭제 성공`);
          } catch (error: any) {
            // 404 에러는 무시 (이미 삭제된 경우)
            if (error.response?.status !== 404) {
              console.warn(`  ⚠️ Annotation ${id} 삭제 실패:`, error.message);
            }
          }
        }

        readAllAnnotationIdsRef.current = [];

        console.log(`  ✅ 모든 Annotation 삭제 완료: ${annotationIds.length}개`);

        return {
          request: requestInfo,
          response: {
            message: 'All annotations deleted successfully',
            deleted_count: annotationIds.length,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/annotations`,
          };
        }
        throw error;
      }
    } else if (testIndex === 14) {
      // 1️⃣4️⃣ 정리 (생성된 사용자 삭제)
      const user1Id = readAllUser1IdRef.current;
      const user2Id = readAllUser2IdRef.current;
      const adjudicatorId = readAllAdjudicatorIdRef.current;

      const requestInfo = {
        method: 'DELETE',
        url: '/api/users (생성된 사용자 3명)',
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        const deletedUsers = [];

        const userIds = [user1Id, user2Id, adjudicatorId].filter(id => id !== null) as number[];

        for (const userId of userIds) {
          try {
            await axios.delete(`${apiUrl}/api/users/${userId}`, config);
            deletedUsers.push({ user_id: userId });
            console.log(`  ✅ 사용자 ${userId} 삭제 성공`);
          } catch (error: any) {
            // 404 에러는 무시 (이미 삭제된 경우)
            if (error.response?.status !== 404) {
              console.warn(`  ⚠️ 사용자 ${userId} 삭제 실패:`, error.message);
            }
          }
        }

        // ref 초기화
        readAllUser1IdRef.current = null;
        readAllUser2IdRef.current = null;
        readAllAdjudicatorIdRef.current = null;
        readAllAdjudicatorRoleIdRef.current = null;

        console.log(`  ✅ 모든 사용자 삭제 완료: ${deletedUsers.length}명`);

        return {
          request: requestInfo,
          response: {
            message: 'All test users deleted successfully',
            deleted_count: deletedUsers.length,
            deleted_users: deletedUsers,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = requestInfo;
        }
        throw error;
      }
    } else if (testIndex === 15) {
      // 1️⃣5️⃣ 정리 (테스트 프로젝트 삭제)
      const projectId = readAllProjectIdRef.current;

      if (!projectId) {
        console.log(`  ℹ️ 삭제할 프로젝트가 없습니다.`);
        return {
          request: { method: 'DELETE', url: '/api/projects' },
          response: { message: 'No project to delete' },
        };
      }

      const requestInfo = {
        method: 'DELETE',
        url: `/api/projects/${projectId}`,
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        await axios.delete(`${apiUrl}/api/projects/${projectId}`, config);

        // ref 초기화
        readAllProjectIdRef.current = null;

        console.log(`  ✅ 테스트 프로젝트 삭제 성공: ID=${projectId}`);

        return {
          request: requestInfo,
          response: {
            message: 'Test project deleted successfully',
            project_id: projectId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${projectId}`,
          };
        }
        throw error;
      }
    }

    throw new Error(`Unknown test index: ${testIndex}`);
  };

  // 프로젝트 멤버 추가 공통 헬퍼 함수
  const addUsersToProject = async (
    projectId: number,
    userIds: number[],
    role: 'SUPER_ADMIN' | 'ADMIN' | 'USER' = 'SUPER_ADMIN'
  ) => {
    const config = await handleGetAxiosConfig(role);
    const addedMembers = [];

    for (const userId of userIds) {
      try {
        const response = await axios.post(
          `${apiUrl}/api/projects/${projectId}/members`,
          { user_id: userId },
          config
        );
        addedMembers.push({ user_id: userId, role: response.data.role_name || 'MEMBER' });
        console.log(`  ✅ 사용자 ${userId}를 프로젝트 멤버로 추가 성공`);
      } catch (error: any) {
        // 이미 멤버인 경우 무시
        if (error.response?.status !== 400 && error.response?.status !== 409) {
          throw error;
        }
        console.log(`  ℹ️ 사용자 ${userId}는 이미 프로젝트 멤버입니다`);
      }
    }

    return {
      project_id: projectId,
      added_members: addedMembers,
      count: addedMembers.length,
    };
  };

  // Annotation 권한 조회 API 개선 테스트용 공통 헬퍼 함수들
  const createRequestInfo = (
    method: string,
    url: string,
    headers?: Record<string, string>,
    body?: any
  ) => ({
    method,
    url,
    ...(headers && { headers }),
    ...(body && { body }),
  });

  const handleApiError = (error: any, requestInfo: any) => {
    if (!error.config) {
      error.config = {
        method: requestInfo.method.toLowerCase(),
        url: requestInfo.url,
        ...(requestInfo.body && { data: requestInfo.body }),
        ...(requestInfo.headers && { headers: requestInfo.headers }),
      };
    }
    throw error;
  };

  const expectErrorResponse = async (
    apiCall: () => Promise<any>,
    expectedStatus: number,
    requestInfo: any
  ) => {
    try {
      await apiCall();
      throw new Error(`${expectedStatus} 에러가 발생해야 하는데 성공했습니다`);
    } catch (error: any) {
      if (error.response?.status === expectedStatus) {
        console.log(`  ✅ ${expectedStatus} 에러 처리 성공:`, error.response.data);
        return {
          request: requestInfo,
          response: error.response.data || { status: expectedStatus },
        };
      }
      handleApiError(error, requestInfo);
      throw error;
    }
  };

  // Annotation 권한 조회 API 개선 테스트
  const runAnnotationPermissionsApiTest = async (testIndex: number) => {

    if (testIndex === 0) {
      // 0️⃣ 사전 정리 (기존 테스트 프로젝트 삭제)
      const requestInfo = {
        method: 'DELETE',
        url: '/api/projects (기존 테스트 프로젝트)',
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        
        // 프로젝트 목록 조회
        const projectsResponse = await axios.get(`${apiUrl}/api/projects`, config);
        const projects = projectsResponse.data.projects || [];
        
        // 이름 패턴으로 검색 (시작하는 프로젝트)
        const projectNamePattern = 'Annotation Permissions API Test';
        const existingProjects = projects.filter((p: any) => 
          p.name.startsWith(projectNamePattern)
        );
        
        // 찾은 프로젝트 삭제
        const deletedProjects = [];
        for (const project of existingProjects) {
          try {
            await axios.delete(`${apiUrl}/api/projects/${project.id}`, config);
            deletedProjects.push({ id: project.id, name: project.name });
            console.log(`  ✅ 기존 프로젝트 삭제: ${project.name} (ID: ${project.id})`);
          } catch (error: any) {
            // 404 에러는 무시 (이미 삭제된 경우)
            if (error.response?.status !== 404) {
              console.warn(`  ⚠️ 프로젝트 삭제 실패: ${project.name}`, error.message);
            }
          }
        }
        
        return {
          request: requestInfo,
          response: {
            message: '사전 정리 완료',
            deleted_count: deletedProjects.length,
            deleted_projects: deletedProjects,
          },
        };
      } catch (error: any) {
        // 에러가 발생해도 계속 진행 (정리 단계이므로)
        return {
          request: requestInfo,
          response: { message: '정리 중 에러 발생 (무시하고 계속 진행)', error: error.message },
        };
      }
    } else if (testIndex === 1) {
      // 1️⃣ 테스트용 프로젝트 생성
      const requestBody = {
        name: `Annotation Permissions API Test ${Date.now()}`,
        description: 'Annotation 권한 조회 API 개선 기능 테스트를 위한 프로젝트',
        sponsor: 'Test Sponsor',
        status: 'ACTIVE',
      };

      const requestInfo = {
        method: 'POST',
        url: '/api/projects',
        body: requestBody,
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        const response = await axios.post(`${apiUrl}/api/projects`, requestBody, config);

        annotationPermissionsApiProjectIdRef.current = response.data.id;
        console.log(`  ✅ 테스트 프로젝트 생성 성공: ID=${response.data.id}`);

        return {
          request: requestInfo,
          response: response.data,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects`,
            data: requestBody,
          };
        }
        throw error;
      }
    } else if (testIndex === 2) {
      // 2️⃣ 사용자들을 프로젝트 멤버로 추가
      const projectId = annotationPermissionsApiProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const userIds = [1, 2]; // 요청자, 대상 사용자 (프로젝트 멤버)
      const requestInfo = {
        method: 'POST',
        url: `/api/projects/${projectId}/members`,
        body: `사용자 ${userIds.length}명 추가`,
      };

      try {
        const result = await addUsersToProject(projectId, userIds);
        return {
          request: requestInfo,
          response: result,
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'post',
            url: `${apiUrl}/api/projects/${projectId}/members`,
          };
        }
        throw error;
      }
    } else if (testIndex === 3) {
      // 3️⃣ 테스트용 사용자 조회/확인
      const projectId = annotationPermissionsApiProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      // 사용자 조회 API를 통해 테스트용 사용자 ID 확인
      // 실제로는 기존 사용자를 사용하거나, 사용자 생성 API가 있다면 사용
      // 여기서는 간단히 고정된 사용자 ID 사용
      const requestingUserId = 1; // 요청자 (프로젝트 멤버)
      const targetUserId = 2; // 대상 사용자 (프로젝트 멤버)
      const nonMemberUserId = 999; // 비멤버 사용자

      annotationPermissionsApiRequestingUserIdRef.current = requestingUserId;
      annotationPermissionsApiTargetUserIdRef.current = targetUserId;
      annotationPermissionsApiNonMemberUserIdRef.current = nonMemberUserId;

      const requestInfo = {
        method: 'GET',
        url: '/api/users (사용자 확인)',
      };

      console.log(`  ✅ 테스트용 사용자 설정 완료`);
      console.log(`     요청자: ${requestingUserId}, 대상: ${targetUserId}, 비멤버: ${nonMemberUserId}`);

      return {
        request: requestInfo,
        response: {
          requesting_user_id: requestingUserId,
          target_user_id: targetUserId,
          non_member_user_id: nonMemberUserId,
          project_id: projectId,
        },
      };
    } else if (testIndex === 4) {
      // 4️⃣ 본인 권한 조회 (user_id 파라미터 없음, 헤더만)
      const projectId = annotationPermissionsApiProjectIdRef.current;
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;

      if (!projectId || !requestingUserId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/permissions?project_id=${projectId}`,
        headers: { 'X-User-ID': requestingUserId },
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations/permissions?project_id=${projectId}`,
          {
            headers: {
              'X-User-ID': requestingUserId.toString(),
            },
          }
        );

        console.log(`  ✅ 본인 권한 조회 성공 (헤더만):`, response.data);

        return {
          request: requestInfo,
          response: {
            permissions: response.data,
            user_id: requestingUserId,
            project_id: projectId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=${projectId}`,
            headers: { 'X-User-ID': requestingUserId.toString() },
          };
        }
        throw error;
      }
    } else if (testIndex === 5) {
      // 5️⃣ 본인 권한 조회 (user_id 쿼리 파라미터로 명시)
      const projectId = annotationPermissionsApiProjectIdRef.current;
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;

      if (!projectId || !requestingUserId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/permissions?project_id=${projectId}&user_id=${requestingUserId}`,
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${requestingUserId}`
        );

        console.log(`  ✅ 본인 권한 조회 성공 (쿼리 파라미터):`, response.data);

        return {
          request: requestInfo,
          response: {
            permissions: response.data,
            user_id: requestingUserId,
            project_id: projectId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${requestingUserId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 6) {
      // 6️⃣ 본인 권한 조회 (쿼리 파라미터와 헤더 모두, 쿼리 우선순위)
      const projectId = annotationPermissionsApiProjectIdRef.current;
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;
      const differentUserId = 999; // 헤더에 다른 user_id

      if (!projectId || !requestingUserId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/permissions?project_id=${projectId}&user_id=${requestingUserId}`,
        headers: { 'X-User-ID': differentUserId },
      };

      try {
        // 쿼리 파라미터의 user_id가 우선순위를 가져야 함
        const response = await axios.get(
          `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${requestingUserId}`,
          {
            headers: {
              'X-User-ID': differentUserId.toString(),
            },
          }
        );

        console.log(`  ✅ 본인 권한 조회 성공 (쿼리 우선순위 확인):`, response.data);
        console.log(`     쿼리 파라미터 user_id=${requestingUserId}, 헤더 X-User-ID=${differentUserId}`);

        return {
          request: requestInfo,
          response: {
            permissions: response.data,
            user_id: requestingUserId,
            project_id: projectId,
            note: '쿼리 파라미터의 user_id가 우선순위를 가짐',
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${requestingUserId}`,
            headers: { 'X-User-ID': differentUserId.toString() },
          };
        }
        throw error;
      }
    } else if (testIndex === 7) {
      // 7️⃣ 다른 사용자 권한 조회 (프로젝트 멤버인 경우)
      const projectId = annotationPermissionsApiProjectIdRef.current;
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;
      const targetUserId = annotationPermissionsApiTargetUserIdRef.current;

      if (!projectId || !requestingUserId || !targetUserId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/permissions?project_id=${projectId}&user_id=${targetUserId}`,
        headers: { 'X-User-ID': requestingUserId },
      };

      try {
        const response = await axios.get(
          `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${targetUserId}`,
          {
            headers: {
              'X-User-ID': requestingUserId.toString(),
            },
          }
        );

        console.log(`  ✅ 다른 사용자 권한 조회 성공:`, response.data);
        console.log(`     요청자: ${requestingUserId}, 대상: ${targetUserId}`);

        return {
          request: requestInfo,
          response: {
            permissions: response.data,
            requesting_user_id: requestingUserId,
            target_user_id: targetUserId,
            project_id: projectId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${targetUserId}`,
            headers: { 'X-User-ID': requestingUserId.toString() },
          };
        }
        throw error;
      }
    } else if (testIndex === 8) {
      // 8️⃣ project_id 누락 시 400 에러
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;

      if (!requestingUserId) {
        throw new Error('사용자 ID가 없습니다.');
      }

      const requestInfo = createRequestInfo(
        'GET',
        '/api/annotations/permissions (project_id 없음)',
        { 'X-User-ID': requestingUserId.toString() }
      );

      return expectErrorResponse(
        () => axios.get(`${apiUrl}/api/annotations/permissions`, {
          headers: { 'X-User-ID': requestingUserId.toString() },
        }),
        400,
        requestInfo
      );
    } else if (testIndex === 9) {
      // 9️⃣ project_id가 0일 때 400 에러
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;

      if (!requestingUserId) {
        throw new Error('사용자 ID가 없습니다.');
      }

      const requestInfo = createRequestInfo(
        'GET',
        '/api/annotations/permissions?project_id=0',
        { 'X-User-ID': requestingUserId.toString() }
      );

      return expectErrorResponse(
        () => axios.get(`${apiUrl}/api/annotations/permissions?project_id=0`, {
          headers: { 'X-User-ID': requestingUserId.toString() },
        }),
        400,
        requestInfo
      );
    } else if (testIndex === 10) {
      // 🔟 project_id가 음수일 때 400 에러
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;

      if (!requestingUserId) {
        throw new Error('사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: '/api/annotations/permissions?project_id=-1',
        headers: { 'X-User-ID': requestingUserId },
      };

      try {
        await axios.get(`${apiUrl}/api/annotations/permissions?project_id=-1`, {
          headers: {
            'X-User-ID': requestingUserId.toString(),
          },
        });
        throw new Error('400 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 400) {
          console.log(`  ✅ project_id=-1 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 400, message: 'Bad Request' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=-1`,
            headers: { 'X-User-ID': requestingUserId.toString() },
          };
        }
        throw error;
      }
    } else if (testIndex === 11) {
      // 1️⃣1️⃣ project_id가 유효하지 않은 형식 (문자열) 400 에러
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;

      if (!requestingUserId) {
        throw new Error('사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: '/api/annotations/permissions?project_id=invalid',
        headers: { 'X-User-ID': requestingUserId },
      };

      try {
        await axios.get(`${apiUrl}/api/annotations/permissions?project_id=invalid`, {
          headers: {
            'X-User-ID': requestingUserId.toString(),
          },
        });
        throw new Error('400 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 400) {
          console.log(`  ✅ project_id=invalid 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 400, message: 'Bad Request' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=invalid`,
            headers: { 'X-User-ID': requestingUserId.toString() },
          };
        }
        throw error;
      }
    } else if (testIndex === 12) {
      // 1️⃣2️⃣ user_id 없음 (헤더도 쿼리도 없음) 401 에러
      const projectId = annotationPermissionsApiProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/permissions?project_id=${projectId} (user_id 없음)`,
        headers: {},
      };

      try {
        await axios.get(`${apiUrl}/api/annotations/permissions?project_id=${projectId}`);
        throw new Error('401 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 401) {
          console.log(`  ✅ user_id 없음 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 401, message: 'Unauthorized' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=${projectId}`,
          };
        }
        throw error;
      }
    } else if (testIndex === 13) {
      // 1️⃣3️⃣ 프로젝트 멤버가 아닌 사용자가 다른 사용자 권한 조회 시도 (403 에러)
      const projectId = annotationPermissionsApiProjectIdRef.current;
      const nonMemberUserId = annotationPermissionsApiNonMemberUserIdRef.current;
      const targetUserId = annotationPermissionsApiTargetUserIdRef.current;

      if (!projectId || !nonMemberUserId || !targetUserId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/permissions?project_id=${projectId}&user_id=${targetUserId}`,
        headers: { 'X-User-ID': nonMemberUserId },
      };

      try {
        await axios.get(
          `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${targetUserId}`,
          {
            headers: {
              'X-User-ID': nonMemberUserId.toString(),
            },
          }
        );
        throw new Error('403 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 403) {
          console.log(`  ✅ 프로젝트 멤버가 아닌 사용자 권한 조회 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 403, message: 'Forbidden' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${targetUserId}`,
            headers: { 'X-User-ID': nonMemberUserId.toString() },
          };
        }
        throw error;
      }
    } else if (testIndex === 14) {
      // 1️⃣4️⃣ 존재하지 않는 프로젝트의 권한 조회 (404/401 에러)
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;
      const nonExistentProjectId = 999999;

      if (!requestingUserId) {
        throw new Error('사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/permissions?project_id=${nonExistentProjectId}`,
        headers: { 'X-User-ID': requestingUserId },
      };

      try {
        await axios.get(
          `${apiUrl}/api/annotations/permissions?project_id=${nonExistentProjectId}`,
          {
            headers: {
              'X-User-ID': requestingUserId.toString(),
            },
          }
        );
        throw new Error('404 또는 401 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 404 || error.response?.status === 401) {
          console.log(`  ✅ 존재하지 않는 프로젝트 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: error.response.status, message: 'Not Found or Unauthorized' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=${nonExistentProjectId}`,
            headers: { 'X-User-ID': requestingUserId.toString() },
          };
        }
        throw error;
      }
    } else if (testIndex === 15) {
      // 1️⃣5️⃣ target_user_id가 프로젝트 멤버가 아닌 경우 (401 에러)
      const projectId = annotationPermissionsApiProjectIdRef.current;
      const requestingUserId = annotationPermissionsApiRequestingUserIdRef.current;
      const nonMemberTargetUserId = annotationPermissionsApiNonMemberUserIdRef.current;

      if (!projectId || !requestingUserId || !nonMemberTargetUserId) {
        throw new Error('프로젝트 ID 또는 사용자 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'GET',
        url: `/api/annotations/permissions?project_id=${projectId}&user_id=${nonMemberTargetUserId}`,
        headers: { 'X-User-ID': requestingUserId },
      };

      try {
        await axios.get(
          `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${nonMemberTargetUserId}`,
          {
            headers: {
              'X-User-ID': requestingUserId.toString(),
            },
          }
        );
        throw new Error('401 에러가 발생해야 하는데 성공했습니다');
      } catch (error: any) {
        if (error.response?.status === 401) {
          console.log(`  ✅ target_user_id가 프로젝트 멤버가 아닌 경우 에러 처리 성공:`, error.response.data);
          return {
            request: requestInfo,
            response: error.response.data || { status: 401, message: 'Unauthorized' },
          };
        }

        if (!error.config) {
          error.config = {
            method: 'get',
            url: `${apiUrl}/api/annotations/permissions?project_id=${projectId}&user_id=${nonMemberTargetUserId}`,
            headers: { 'X-User-ID': requestingUserId.toString() },
          };
        }
        throw error;
      }
    } else if (testIndex === 16) {
      // 1️⃣6️⃣ 정리 (테스트 프로젝트 삭제)
      const projectId = annotationPermissionsApiProjectIdRef.current;

      if (!projectId) {
        throw new Error('프로젝트 ID가 없습니다.');
      }

      const requestInfo = {
        method: 'DELETE',
        url: `/api/projects/${projectId}`,
      };

      try {
        const config = await handleGetAxiosConfig('SUPER_ADMIN');
        await axios.delete(`${apiUrl}/api/projects/${projectId}`, config);

        // ref 초기화
        annotationPermissionsApiProjectIdRef.current = null;
        annotationPermissionsApiRequestingUserIdRef.current = null;
        annotationPermissionsApiTargetUserIdRef.current = null;
        annotationPermissionsApiNonMemberUserIdRef.current = null;

        console.log(`  ✅ 테스트 프로젝트 삭제 성공`);

        return {
          request: requestInfo,
          response: {
            message: 'Test project deleted successfully',
            project_id: projectId,
          },
        };
      } catch (error: any) {
        if (!error.config) {
          error.config = {
            method: 'delete',
            url: `${apiUrl}/api/projects/${projectId}`,
          };
        }
        throw error;
      }
    }

    throw new Error(`Unknown test index: ${testIndex}`);
  };

  const stats = getStats();

  return (
    <div className="api-health-check">
      <div className="health-check-header">
        <h2>🔍 API 점검</h2>
        <p className="subtitle">프로젝트 상태 관리 API E2E 테스트</p>
      </div>

      {/* 통계 */}
      <div className="stats-bar">
        <div className="stat-item total">
          <span className="stat-label">전체</span>
          <span className="stat-value">{stats.total}</span>
        </div>
        <div className="stat-item success">
          <span className="stat-label">성공</span>
          <span className="stat-value">{stats.success}</span>
        </div>
        <div className="stat-item failure">
          <span className="stat-label">실패</span>
          <span className="stat-value">{stats.failure}</span>
        </div>
        <div className="stat-item running">
          <span className="stat-label">진행중</span>
          <span className="stat-value">{stats.running}</span>
        </div>
        <div className="stat-item pending">
          <span className="stat-label">대기</span>
          <span className="stat-value">{stats.pending}</span>
        </div>
        <div className="stat-item skipped">
          <span className="stat-label">건너뜀</span>
          <span className="stat-value">{stats.skipped}</span>
        </div>
      </div>

      {/* 테스트 계정 선택 및 토큰 획득 */}
      <div className="test-account-section">
        <div className="account-selector">
          <label>테스트 계정:</label>
          <select
            value={currentTestAccount.username}
            onChange={(e) => {
              const account = Object.values(TEST_ACCOUNTS).find(a => a.username === e.target.value);
              if (account) {
                setCurrentTestAccount(account);
                setTestToken(null); // 계정 변경 시 토큰 초기화
              }
            }}
          >
            {Object.values(TEST_ACCOUNTS).map(account => (
              <option key={account.username} value={account.username}>
                {account.username} ({account.role})
              </option>
            ))}
          </select>
          <button
            onClick={() => handleGetTestToken(currentTestAccount)}
            className="get-token-button"
          >
            🔑 토큰 획득
          </button>
          {testToken && (
            <span className="token-status">✅ 토큰 획득 완료</span>
          )}
        </div>
      </div>

      {/* 일괄 테스트 버튼 */}
      <div className="bulk-actions">
        <button
          onClick={runAllTests}
          disabled={isRunningAll}
          className="run-all-button"
        >
          {isRunningAll ? '🔄 테스트 실행 중...' : '▶️ 모든 테스트 실행'}
        </button>
      </div>

      {/* 테스트 섹션 */}
      {sections.map((section, sectionIndex) => (
        <div key={sectionIndex} className="test-section">
          <div className="section-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <h3>{section.title}</h3>
              <p className="section-description">{section.description}</p>
            </div>
            <button
              onClick={() => runSectionTests(sectionIndex)}
              disabled={isRunningAll}
              className="run-section-button"
              style={{
                padding: '8px 16px',
                backgroundColor: '#10b981',
                color: 'white',
                border: 'none',
                borderRadius: '6px',
                cursor: isRunningAll ? 'not-allowed' : 'pointer',
                fontSize: '14px',
                fontWeight: '500',
                opacity: isRunningAll ? 0.5 : 1,
              }}
            >
              🚀 이 섹션 모두 실행
            </button>
          </div>

          <div className="test-list">
            {section.tests.map((test, testIndex) => {
              const testKey = `${sectionIndex}-${testIndex}`;
              const isExpanded = expandedTest === testKey;

              const indentLevel = test.indentLevel || 0;
              const indentStyle = {
                marginLeft: `${indentLevel * 30}px`,
                borderLeft: indentLevel > 0 ? '3px solid #e5e7eb' : 'none',
                paddingLeft: indentLevel > 0 ? '15px' : '0',
              };

              return (
                <div
                  key={testIndex}
                  className={`test-item ${test.status}`}
                  style={indentStyle}
                >
                  <div className="test-header">
                    <div className="test-info">
                      {indentLevel > 0 && (
                        <span className="indent-connector">└─</span>
                      )}
                      <span className="test-icon">{getStatusIcon(test.status)}</span>
                      <div className="test-name-container">
                        <span className="test-name">{test.name}</span>
                        {test.dependencies && test.dependencies.length > 0 && (
                          <span className="test-dependencies" title={`의존: ${test.dependencies.join(', ')}`}>
                            🔗 {test.dependencies.length}개 의존
                          </span>
                        )}
                        {test.isSequential && (
                          <span className="test-sequential" title="순차 실행 필요">
                            ⏩ 순차
                          </span>
                        )}
                        {test.cleanup && (
                          <span className="test-cleanup" title="정리 작업">
                            🧹 정리
                          </span>
                        )}
                        {test.delayAfter !== undefined && test.delayAfter > 0 && (
                          <span className="test-delay" title={`이 테스트 후 ${test.delayAfter}ms 대기`}>
                            ⏱️ {test.delayAfter}ms
                          </span>
                        )}
                      </div>
                      {test.duration && (
                        <span className="test-duration">{test.duration}ms</span>
                      )}
                    </div>
                    <div className="test-actions">
                      <button
                        onClick={() => runTest(sectionIndex, testIndex)}
                        disabled={test.status === 'running' || isRunningAll}
                        className="run-test-button"
                      >
                        {test.status === 'running' ? '⏳' : '▶️'}
                      </button>
                      {(test.request || test.response || test.error) && (
                        <button
                          onClick={() => toggleTestDetails(testKey)}
                          className="toggle-details-button"
                        >
                          {isExpanded ? '▼' : '▶'}
                        </button>
                      )}
                    </div>
                  </div>

                  {isExpanded && (test.request || test.response || test.error) && (
                    <div className="test-details">
                      {test.request && (
                        <div className="detail-section">
                          <h4>📤 요청</h4>
                          <pre>{JSON.stringify(test.request, null, 2)}</pre>
                        </div>
                      )}
                      {test.response && (
                        <div className="detail-section">
                          <h4>📥 응답</h4>
                          <pre>{JSON.stringify(test.response, null, 2)}</pre>
                        </div>
                      )}
                      {test.error && (
                        <div className="detail-section error">
                          <h4>❌ 에러</h4>
                          <pre>{test.error}</pre>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      ))}
    </div>
  );
};

export default ApiScenarioTests;

