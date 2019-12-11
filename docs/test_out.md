TraceData 0 items :
    !0 #SOME 5
TraceData 0 ops :
    $0 whnf (21) 21 [$1]
    $1 $0 whnf_core (21, !0) 21 []


TraceData 1 items :
    !0 #SOME 5
TraceData 1 ops :
    $0 whnf (18) 18 [$1]
    $1 $0 whnf_core (18, !0) 18 []


TraceData 2 items :
    !0 #SOME 5
TraceData 2 ops :
    $0 whnf (35) 35 [$1]
    $1 $0 whnf_core (35, !0) 35 []


TraceData 3 items :
    !0 #SOME 5
TraceData 3 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 4 items :
    !0 #SOME 5
TraceData 4 ops :
    $0 whnf (35) 35 [$1]
    $1 $0 whnf_core (35, !0) 35 []


TraceData 5 items :
    !0 #SOME 5
TraceData 5 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 6 items :
    !0 #ELO #BI 0 12 14
    !1 #EA 23 !0
    !2 #ELO #BD 1 15 !0
    !3 #EA !1 !2
    !4 #EA !3 !2
    !5 #SOME 5
TraceData 6 ops :
    $0 whnf (!4) !4 [$1]
    $1 $0 whnf_core (!4, !5) !4 []


TraceData 7 items :
    !0 #ELO #BI 0 12 14
    !1 #EA 23 !0
    !2 #ELO #BD 1 15 !0
    !3 #EA !1 !2
TraceData 7 ops :
    $0 check_def_eq (!3, !3) 3 []


TraceData 8 items :
    !0 #ELO #BD 12 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #EA 41 !0
    !4 #EP #BD 37 !2 !3
    !5 #SOME 5
TraceData 8 ops :
    $0 whnf (!4) !4 [$1]
    $1 $0 whnf_core (!4, !5) !4 []


TraceData 9 items :
    !0 #ELO #BD 12 12 34
    !1 #EA 41 !0
    !2 #SOME 5
TraceData 9 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 10 items :
    !0 #ELO #BD 12 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #SOME 5
TraceData 10 ops :
    $0 whnf (!2) !2 [$1]
    $1 $0 whnf_core (!2, !3) !2 []


TraceData 11 items :
    !0 #ELO #BD 12 12 34
    !1 #SOME 5
TraceData 11 ops :
    $0 whnf (!0) !0 [$1]
    $1 $0 whnf_core (!0, !1) !0 []


TraceData 12 items :
    !0 #ELO #BD 12 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #ELO #BD 17 37 !2
TraceData 12 ops :
    $0 infer (!3) !2 []


TraceData 14 items :
    !0 #ELO #BD 12 12 34
TraceData 14 ops :
    $0 infer (!0) 34 []


TraceData 15 items :
TraceData 15 ops :
    $0 whnf (34) 34 []


TraceData 16 items :
    !0 #ELO #BD 12 12 34
TraceData 16 ops :
    $0 infer (!0) 34 []


TraceData 17 items :
TraceData 17 ops :
    $0 whnf (34) 34 []


TraceData 18 items :
    !0 #ELO #BD 12 12 34
TraceData 18 ops :
    $0 infer (!0) 34 []


TraceData 19 items :
TraceData 19 ops :
    $0 whnf (34) 34 []


TraceData 13 items :
    !0 #ELO #BD 12 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 13 ops :
    $0 infer (!2) !5 []


TraceData 20 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 13
    !5 #UM 13 !4
    !6 #US !5
    !7 #ES !6
TraceData 20 ops :
    $0 whnf (!2) !7 [$1]
    $1 $0 whnf_core (!2, !3) !7 []


TraceData 21 items :
    !0 #ELO #BD 12 12 34
    !1 #EA 41 !0
TraceData 21 ops :
    $0 check_def_eq (!1, !1) 3 []


TraceData 22 items :
    !0 #ELO #BD 12 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #ELO #BD 17 37 !2
TraceData 22 ops :
    $0 infer (!3) !2 []


TraceData 24 items :
    !0 #ELO #BD 12 12 34
TraceData 24 ops :
    $0 infer (!0) 34 []


TraceData 25 items :
    !0 #SOME 5
TraceData 25 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 26 items :
    !0 #ELO #BD 12 12 34
TraceData 26 ops :
    $0 infer (!0) 34 []


TraceData 27 items :
TraceData 27 ops :
    $0 whnf (34) 34 []


TraceData 28 items :
    !0 #ELO #BD 12 12 34
TraceData 28 ops :
    $0 infer (!0) 34 []


TraceData 29 items :
TraceData 29 ops :
    $0 whnf (34) 34 []


TraceData 23 items :
    !0 #ELO #BD 12 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 23 ops :
    $0 infer (!2) !5 []


TraceData 30 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 13
    !5 #UM 13 !4
    !6 #US !5
    !7 #ES !6
TraceData 30 ops :
    $0 whnf (!2) !7 [$1]
    $1 $0 whnf_core (!2, !3) !7 []


TraceData 31 items :
    !0 #ELO #BD 12 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #ELO #BD 17 37 !2
TraceData 31 ops :
    $0 infer (!3) !2 []


TraceData 32 items :
    !0 #ELO #BD 12 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 32 ops :
    $0 infer (!2) !5 []


TraceData 33 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #UM 13 13
    !4 #UM 13 !3
    !5 #US !4
    !6 #ES !5
TraceData 33 ops :
    $0 whnf (!2) !6 []


TraceData 34 items :
    !0 #ELO #BD 13 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #EA 70 !0
    !4 #EA 72 !0
    !5 #EA 74 !0
    !6 #EA !5 47
    !7 #EA !4 !6
    !8 #EA !7 38
    !9 #EA !8 17
    !10 #EA !7 !9
    !11 #EA !10 16
    !12 #EA !3 !11
    !13 #EA !7 17
    !14 #EA !13 16
    !15 #EA !8 !14
    !16 #EA !12 !15
    !17 #EP #BD 45 !0 !16
    !18 #EP #BD 69 !0 !17
    !19 #EP #BD 15 !0 !18
    !20 #EA 90 !0
    !21 #EP #BD 68 !19 !20
    !22 #EP #BD 37 !2 !21
    !23 #SOME 5
TraceData 34 ops :
    $0 whnf (!22) !22 [$1]
    $1 $0 whnf_core (!22, !23) !22 []


TraceData 35 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 90 !0
    !2 #SOME 5
TraceData 35 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 36 items :
    !0 #ELO #BD 13 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #SOME 5
TraceData 36 ops :
    $0 whnf (!2) !2 [$1]
    $1 $0 whnf_core (!2, !3) !2 []


TraceData 37 items :
    !0 #ELO #BD 13 12 34
    !1 #SOME 5
TraceData 37 ops :
    $0 whnf (!0) !0 [$1]
    $1 $0 whnf_core (!0, !1) !0 []


TraceData 38 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 28 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #EA !8 38
    !10 #EA !9 17
    !11 #EA !8 !10
    !12 #EA !11 16
    !13 #EA !1 !12
    !14 #EA !8 17
    !15 #EA !14 16
    !16 #EA !9 !15
    !17 #EA !13 !16
    !18 #EP #BD 45 !0 !17
    !19 #EP #BD 69 !0 !18
    !20 #EP #BD 15 !0 !19
    !21 #SOME 5
TraceData 38 ops :
    $0 whnf (!20) !20 [$1]
    $1 $0 whnf_core (!20, !21) !20 []


TraceData 39 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 28 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #ELO #BD 32 15 !0
    !10 #EA !8 !9
    !11 #ELO #BD 33 69 !0
    !12 #EA !10 !11
    !13 #EA !8 !12
    !14 #ELO #BD 34 45 !0
    !15 #EA !13 !14
    !16 #EA !1 !15
    !17 #EA !8 !11
    !18 #EA !17 !14
    !19 #EA !10 !18
    !20 #EA !16 !19
    !21 #SOME 5
TraceData 39 ops :
    $0 whnf (!20) !20 [$1]
    $1 $0 whnf_core (!20, !21) !20 []


TraceData 40 items :
    !0 #ELO #BD 13 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #ELO #BD 28 37 !2
TraceData 40 ops :
    $0 infer (!3) !2 []


TraceData 42 items :
    !0 #ELO #BD 13 12 34
TraceData 42 ops :
    $0 infer (!0) 34 []


TraceData 43 items :
TraceData 43 ops :
    $0 whnf (34) 34 []


TraceData 44 items :
    !0 #ELO #BD 13 12 34
TraceData 44 ops :
    $0 infer (!0) 34 []


TraceData 45 items :
TraceData 45 ops :
    $0 whnf (34) 34 []


TraceData 46 items :
    !0 #ELO #BD 13 12 34
TraceData 46 ops :
    $0 infer (!0) 34 []


TraceData 47 items :
TraceData 47 ops :
    $0 whnf (34) 34 []


TraceData 41 items :
    !0 #ELO #BD 13 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 41 ops :
    $0 infer (!2) !5 []


TraceData 48 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 13
    !5 #UM 13 !4
    !6 #US !5
    !7 #ES !6
TraceData 48 ops :
    $0 whnf (!2) !7 [$1]
    $1 $0 whnf_core (!2, !3) !7 []


TraceData 49 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 90 !0
TraceData 49 ops :
    $0 check_def_eq (!1, !1) 3 []


TraceData 50 items :
    !0 #ELO #BD 13 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #ELO #BD 28 37 !2
TraceData 50 ops :
    $0 infer (!3) !2 []


TraceData 52 items :
    !0 #ELO #BD 13 12 34
TraceData 52 ops :
    $0 infer (!0) 34 []


TraceData 53 items :
    !0 #SOME 5
TraceData 53 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 54 items :
    !0 #ELO #BD 13 12 34
TraceData 54 ops :
    $0 infer (!0) 34 []


TraceData 55 items :
TraceData 55 ops :
    $0 whnf (34) 34 []


TraceData 56 items :
    !0 #ELO #BD 13 12 34
TraceData 56 ops :
    $0 infer (!0) 34 []


TraceData 57 items :
TraceData 57 ops :
    $0 whnf (34) 34 []


TraceData 51 items :
    !0 #ELO #BD 13 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 51 ops :
    $0 infer (!2) !5 []


TraceData 58 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 13
    !5 #UM 13 !4
    !6 #US !5
    !7 #ES !6
TraceData 58 ops :
    $0 whnf (!2) !7 [$1]
    $1 $0 whnf_core (!2, !3) !7 []


TraceData 59 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 28 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #EA !8 38
    !10 #EA !9 17
    !11 #EA !8 !10
    !12 #EA !11 16
    !13 #EA !1 !12
    !14 #EA !8 17
    !15 #EA !14 16
    !16 #EA !9 !15
    !17 #EA !13 !16
    !18 #EP #BD 45 !0 !17
    !19 #EP #BD 69 !0 !18
    !20 #EP #BD 15 !0 !19
    !21 #ELO #BD 29 68 !20
TraceData 59 ops :
    $0 infer (!21) !20 []


TraceData 61 items :
    !0 #ELO #BD 13 12 34
TraceData 61 ops :
    $0 infer (!0) 34 []


TraceData 62 items :
TraceData 62 ops :
    $0 whnf (34) 34 []


TraceData 63 items :
    !0 #ELO #BD 13 12 34
TraceData 63 ops :
    $0 infer (!0) 34 []


TraceData 64 items :
TraceData 64 ops :
    $0 whnf (34) 34 []


TraceData 65 items :
    !0 #ELO #BD 13 12 34
TraceData 65 ops :
    $0 infer (!0) 34 []


TraceData 66 items :
TraceData 66 ops :
    $0 whnf (34) 34 []


TraceData 67 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 28 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #ELO #BD 43 15 !0
    !10 #EA !8 !9
    !11 #ELO #BD 44 69 !0
    !12 #EA !10 !11
    !13 #EA !8 !12
    !14 #ELO #BD 45 45 !0
    !15 #EA !13 !14
    !16 #EA !1 !15
    !17 #EA !8 !11
    !18 #EA !17 !14
    !19 #EA !10 !18
    !20 #EA !16 !19
    !21 #EP #BI 12 34 20
    !22 #EA 41 !0
TraceData 67 ops :
    $0 infer (!20) 18 [$1]
    $1 $0 infer_apps (!20) 18 [$2, $3, $6, $45]
    $2 $1 infer (70) !21 []
    $3 $1 check_type (!0, 34) 9 [$4, $5]
    $4 $3 infer (!0) 34 []
    $5 $3 check_def_eq (34, 34) 3 []
    $6 $1 check_type (!15, !0) 9 [$7, $44]
    $7 $6 infer (!15) !0 [$8]
    $8 $7 infer_apps (!15) !0 [$9, $10, $13, $24, $41]
    $9 $8 infer (72) 51 []
    $10 $8 check_type (!0, 34) 9 [$11, $12]
    $11 $10 infer (!0) 34 []
    $12 $10 check_def_eq (34, 34) 3 []
    $13 $8 check_type (!7, !22) 9 [$14, $23]
    $14 $13 infer (!7) !22 [$15]
    $15 $14 infer_apps (!7) !22 [$16, $17, $20]
    $16 $15 infer (74) 44 []
    $17 $15 check_type (!0, 34) 9 [$18, $19]
    $18 $17 infer (!0) 34 []
    $19 $17 check_def_eq (34, 34) 3 []
    $20 $15 check_type (!6, !5) 9 [$21, $22]
    $21 $20 infer (!6) !5 []
    $22 $20 check_def_eq (!5, !5) 3 []
    $23 $13 check_def_eq (!22, !22) 3 []
    $24 $8 check_type (!12, !0) 9 [$25, $40]
    $25 $24 infer (!12) !0 [$26]
    $26 $25 infer_apps (!12) !0 [$27, $28, $31, $34, $37]
    $27 $26 infer (72) 51 []
    $28 $26 check_type (!0, 34) 9 [$29, $30]
    $29 $28 infer (!0) 34 []
    $30 $28 check_def_eq (34, 34) 3 []
    $31 $26 check_type (!7, !22) 9 [$32, $33]
    $32 $31 infer (!7) !22 []
    $33 $31 check_def_eq (!22, !22) 3 []
    $34 $26 check_type (!9, !0) 9 [$35, $36]
    $35 $34 infer (!9) !0 []
    $36 $34 check_def_eq (!0, !0) 3 []
    $37 $26 check_type (!11, !0) 9 [$38, $39]
    $38 $37 infer (!11) !0 []
    $39 $37 check_def_eq (!0, !0) 3 []
    $40 $24 check_def_eq (!0, !0) 3 []
    $41 $8 check_type (!14, !0) 9 [$42, $43]
    $42 $41 infer (!14) !0 []
    $43 $41 check_def_eq (!0, !0) 3 []
    $44 $6 check_def_eq (!0, !0) 3 []
    $45 $1 check_type (!19, !0) 9 [$46, $75]
    $46 $45 infer (!19) !0 [$47]
    $47 $46 infer_apps (!19) !0 [$48, $49, $52, $55, $58]
    $48 $47 infer (72) 51 []
    $49 $47 check_type (!0, 34) 9 [$50, $51]
    $50 $49 infer (!0) 34 []
    $51 $49 check_def_eq (34, 34) 3 []
    $52 $47 check_type (!7, !22) 9 [$53, $54]
    $53 $52 infer (!7) !22 []
    $54 $52 check_def_eq (!22, !22) 3 []
    $55 $47 check_type (!9, !0) 9 [$56, $57]
    $56 $55 infer (!9) !0 []
    $57 $55 check_def_eq (!0, !0) 3 []
    $58 $47 check_type (!18, !0) 9 [$59, $74]
    $59 $58 infer (!18) !0 [$60]
    $60 $59 infer_apps (!18) !0 [$61, $62, $65, $68, $71]
    $61 $60 infer (72) 51 []
    $62 $60 check_type (!0, 34) 9 [$63, $64]
    $63 $62 infer (!0) 34 []
    $64 $62 check_def_eq (34, 34) 3 []
    $65 $60 check_type (!7, !22) 9 [$66, $67]
    $66 $65 infer (!7) !22 []
    $67 $65 check_def_eq (!22, !22) 3 []
    $68 $60 check_type (!11, !0) 9 [$69, $70]
    $69 $68 infer (!11) !0 []
    $70 $68 check_def_eq (!0, !0) 3 []
    $71 $60 check_type (!14, !0) 9 [$72, $73]
    $72 $71 infer (!14) !0 []
    $73 $71 check_def_eq (!0, !0) 3 []
    $74 $58 check_def_eq (!0, !0) 3 []
    $75 $45 check_def_eq (!0, !0) 3 []


TraceData 68 items :
    !0 #SOME 5
TraceData 68 ops :
    $0 whnf (18) 18 [$1]
    $1 $0 whnf_core (18, !0) 18 []


TraceData 60 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 28 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #EA !8 38
    !10 #EA !9 17
    !11 #EA !8 !10
    !12 #EA !11 16
    !13 #EA !1 !12
    !14 #EA !8 17
    !15 #EA !14 16
    !16 #EA !9 !15
    !17 #EA !13 !16
    !18 #EP #BD 45 !0 !17
    !19 #EP #BD 69 !0 !18
    !20 #EP #BD 15 !0 !19
    !21 #UIM 33 1
    !22 #UIM 33 !21
    !23 #UIM 33 !22
    !24 #ES !23
TraceData 60 ops :
    $0 infer (!20) !24 []


TraceData 69 items :
    !0 #UIM 33 1
    !1 #UIM 33 !0
    !2 #UIM 33 !1
    !3 #ES !2
    !4 #SOME 5
TraceData 69 ops :
    $0 whnf (!3) 18 [$1]
    $1 $0 whnf_core (!3, !4) 18 []


TraceData 70 items :
    !0 #ELO #BD 13 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #ELO #BD 28 37 !2
TraceData 70 ops :
    $0 infer (!3) !2 []


TraceData 71 items :
    !0 #ELO #BD 13 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 71 ops :
    $0 infer (!2) !5 []


TraceData 72 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #UM 13 13
    !4 #UM 13 !3
    !5 #US !4
    !6 #ES !5
TraceData 72 ops :
    $0 whnf (!2) !6 []


TraceData 73 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 28 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #EA !8 38
    !10 #EA !9 17
    !11 #EA !8 !10
    !12 #EA !11 16
    !13 #EA !1 !12
    !14 #EA !8 17
    !15 #EA !14 16
    !16 #EA !9 !15
    !17 #EA !13 !16
    !18 #EP #BD 45 !0 !17
    !19 #EP #BD 69 !0 !18
    !20 #EP #BD 15 !0 !19
    !21 #ELO #BD 29 68 !20
TraceData 73 ops :
    $0 infer (!21) !20 []


TraceData 75 items :
    !0 #ELO #BD 13 12 34
TraceData 75 ops :
    $0 infer (!0) 34 []


TraceData 76 items :
TraceData 76 ops :
    $0 whnf (34) 34 []


TraceData 77 items :
    !0 #ELO #BD 13 12 34
TraceData 77 ops :
    $0 infer (!0) 34 []


TraceData 78 items :
TraceData 78 ops :
    $0 whnf (34) 34 []


TraceData 79 items :
    !0 #ELO #BD 13 12 34
TraceData 79 ops :
    $0 infer (!0) 34 []


TraceData 80 items :
TraceData 80 ops :
    $0 whnf (34) 34 []


TraceData 81 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 28 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #ELO #BD 46 15 !0
    !10 #EA !8 !9
    !11 #ELO #BD 47 69 !0
    !12 #EA !10 !11
    !13 #EA !8 !12
    !14 #ELO #BD 48 45 !0
    !15 #EA !13 !14
    !16 #EA !1 !15
    !17 #EA !8 !11
    !18 #EA !17 !14
    !19 #EA !10 !18
    !20 #EA !16 !19
    !21 #EP #BI 12 34 20
    !22 #EA 41 !0
TraceData 81 ops :
    $0 infer (!20) 18 [$1]
    $1 $0 infer_apps (!20) 18 [$2, $3, $6, $45]
    $2 $1 infer (70) !21 []
    $3 $1 check_type (!0, 34) 9 [$4, $5]
    $4 $3 infer (!0) 34 []
    $5 $3 check_def_eq (34, 34) 3 []
    $6 $1 check_type (!15, !0) 9 [$7, $44]
    $7 $6 infer (!15) !0 [$8]
    $8 $7 infer_apps (!15) !0 [$9, $10, $13, $24, $41]
    $9 $8 infer (72) 51 []
    $10 $8 check_type (!0, 34) 9 [$11, $12]
    $11 $10 infer (!0) 34 []
    $12 $10 check_def_eq (34, 34) 3 []
    $13 $8 check_type (!7, !22) 9 [$14, $23]
    $14 $13 infer (!7) !22 [$15]
    $15 $14 infer_apps (!7) !22 [$16, $17, $20]
    $16 $15 infer (74) 44 []
    $17 $15 check_type (!0, 34) 9 [$18, $19]
    $18 $17 infer (!0) 34 []
    $19 $17 check_def_eq (34, 34) 3 []
    $20 $15 check_type (!6, !5) 9 [$21, $22]
    $21 $20 infer (!6) !5 []
    $22 $20 check_def_eq (!5, !5) 3 []
    $23 $13 check_def_eq (!22, !22) 3 []
    $24 $8 check_type (!12, !0) 9 [$25, $40]
    $25 $24 infer (!12) !0 [$26]
    $26 $25 infer_apps (!12) !0 [$27, $28, $31, $34, $37]
    $27 $26 infer (72) 51 []
    $28 $26 check_type (!0, 34) 9 [$29, $30]
    $29 $28 infer (!0) 34 []
    $30 $28 check_def_eq (34, 34) 3 []
    $31 $26 check_type (!7, !22) 9 [$32, $33]
    $32 $31 infer (!7) !22 []
    $33 $31 check_def_eq (!22, !22) 3 []
    $34 $26 check_type (!9, !0) 9 [$35, $36]
    $35 $34 infer (!9) !0 []
    $36 $34 check_def_eq (!0, !0) 3 []
    $37 $26 check_type (!11, !0) 9 [$38, $39]
    $38 $37 infer (!11) !0 []
    $39 $37 check_def_eq (!0, !0) 3 []
    $40 $24 check_def_eq (!0, !0) 3 []
    $41 $8 check_type (!14, !0) 9 [$42, $43]
    $42 $41 infer (!14) !0 []
    $43 $41 check_def_eq (!0, !0) 3 []
    $44 $6 check_def_eq (!0, !0) 3 []
    $45 $1 check_type (!19, !0) 9 [$46, $75]
    $46 $45 infer (!19) !0 [$47]
    $47 $46 infer_apps (!19) !0 [$48, $49, $52, $55, $58]
    $48 $47 infer (72) 51 []
    $49 $47 check_type (!0, 34) 9 [$50, $51]
    $50 $49 infer (!0) 34 []
    $51 $49 check_def_eq (34, 34) 3 []
    $52 $47 check_type (!7, !22) 9 [$53, $54]
    $53 $52 infer (!7) !22 []
    $54 $52 check_def_eq (!22, !22) 3 []
    $55 $47 check_type (!9, !0) 9 [$56, $57]
    $56 $55 infer (!9) !0 []
    $57 $55 check_def_eq (!0, !0) 3 []
    $58 $47 check_type (!18, !0) 9 [$59, $74]
    $59 $58 infer (!18) !0 [$60]
    $60 $59 infer_apps (!18) !0 [$61, $62, $65, $68, $71]
    $61 $60 infer (72) 51 []
    $62 $60 check_type (!0, 34) 9 [$63, $64]
    $63 $62 infer (!0) 34 []
    $64 $62 check_def_eq (34, 34) 3 []
    $65 $60 check_type (!7, !22) 9 [$66, $67]
    $66 $65 infer (!7) !22 []
    $67 $65 check_def_eq (!22, !22) 3 []
    $68 $60 check_type (!11, !0) 9 [$69, $70]
    $69 $68 infer (!11) !0 []
    $70 $68 check_def_eq (!0, !0) 3 []
    $71 $60 check_type (!14, !0) 9 [$72, $73]
    $72 $71 infer (!14) !0 []
    $73 $71 check_def_eq (!0, !0) 3 []
    $74 $58 check_def_eq (!0, !0) 3 []
    $75 $45 check_def_eq (!0, !0) 3 []


TraceData 82 items :
    !0 #SOME 5
TraceData 82 ops :
    $0 whnf (18) 18 [$1]
    $1 $0 whnf_core (18, !0) 18 []


TraceData 74 items :
    !0 #ELO #BD 13 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 28 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #EA !8 38
    !10 #EA !9 17
    !11 #EA !8 !10
    !12 #EA !11 16
    !13 #EA !1 !12
    !14 #EA !8 17
    !15 #EA !14 16
    !16 #EA !9 !15
    !17 #EA !13 !16
    !18 #EP #BD 45 !0 !17
    !19 #EP #BD 69 !0 !18
    !20 #EP #BD 15 !0 !19
    !21 #UIM 33 1
    !22 #UIM 33 !21
    !23 #UIM 33 !22
    !24 #ES !23
TraceData 74 ops :
    $0 infer (!20) !24 []


TraceData 83 items :
    !0 #UIM 33 1
    !1 #UIM 33 !0
    !2 #UIM 33 !1
    !3 #ES !2
    !4 #SOME 5
TraceData 83 ops :
    $0 whnf (!3) 18 [$1]
    $1 $0 whnf_core (!3, !4) 18 []


TraceData 85 items :
TraceData 85 ops :
    $0 infer (14) 34 []


TraceData 86 items :
    !0 #SOME 5
TraceData 86 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 87 items :
    !0 #ELO #BI 49 12 14
TraceData 87 ops :
    $0 infer (!0) 14 []


TraceData 88 items :
    !0 #SOME 5
TraceData 88 ops :
    $0 whnf (14) 14 [$1]
    $1 $0 whnf_core (14, !0) 14 []


TraceData 89 items :
    !0 #ELO #BI 49 12 14
TraceData 89 ops :
    $0 infer (!0) 14 []


TraceData 90 items :
TraceData 90 ops :
    $0 whnf (14) 14 []


TraceData 91 items :
    !0 #US 1
    !1 #ES !0
TraceData 91 ops :
    $0 infer (18) !1 []


TraceData 92 items :
    !0 #US 1
    !1 #ES !0
    !2 #SOME 5
TraceData 92 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 84 items :
    !0 #US 1
    !1 #UIM 13 !0
    !2 #UIM 13 !1
    !3 #UIM 33 !2
    !4 #ES !3
TraceData 84 ops :
    $0 infer (21) !4 []


TraceData 93 items :
    !0 #US 1
    !1 #UIM 13 !0
    !2 #UIM 13 !1
    !3 #UIM 33 !2
    !4 #ES !3
    !5 #SOME 5
    !6 #UM 13 !0
    !7 #UIM 13 !6
    !8 #UIM 33 !7
    !9 #ES !8
TraceData 93 ops :
    $0 whnf (!4) !9 [$1]
    $1 $0 whnf_core (!4, !5) !9 []


TraceData 95 items :
TraceData 95 ops :
    $0 infer (14) 34 []


TraceData 96 items :
    !0 #SOME 5
TraceData 96 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 97 items :
    !0 #ELO #BI 52 12 14
TraceData 97 ops :
    $0 infer (!0) 14 []


TraceData 98 items :
    !0 #SOME 5
TraceData 98 ops :
    $0 whnf (14) 14 [$1]
    $1 $0 whnf_core (14, !0) 14 []


TraceData 99 items :
    !0 #ELO #BI 52 12 14
    !1 #EA 23 !0
    !2 #ELO #BD 53 15 !0
    !3 #EA !1 !2
    !4 #EA !3 !2
TraceData 99 ops :
    $0 infer (!4) 18 [$1]
    $1 $0 infer_apps (!4) 18 [$2, $3, $6, $9]
    $2 $1 infer (23) 21 []
    $3 $1 check_type (!0, 14) 9 [$4, $5]
    $4 $3 infer (!0) 14 []
    $5 $3 check_def_eq (14, 14) 3 []
    $6 $1 check_type (!2, !0) 9 [$7, $8]
    $7 $6 infer (!2) !0 []
    $8 $6 check_def_eq (!0, !0) 3 []
    $9 $1 check_type (!2, !0) 9 [$10, $11]
    $10 $9 infer (!2) !0 []
    $11 $9 check_def_eq (!0, !0) 3 []


TraceData 100 items :
    !0 #SOME 5
TraceData 100 ops :
    $0 whnf (18) 18 [$1]
    $1 $0 whnf_core (18, !0) 18 []


TraceData 94 items :
    !0 #UIM 13 1
    !1 #UIM 33 !0
    !2 #ES !1
TraceData 94 ops :
    $0 infer (28) !2 []


TraceData 101 items :
    !0 #UIM 13 1
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #SOME 5
TraceData 101 ops :
    $0 whnf (!2) 18 [$1]
    $1 $0 whnf_core (!2, !3) 18 []


TraceData 103 items :
TraceData 103 ops :
    $0 infer (14) 34 []


TraceData 104 items :
    !0 #SOME 5
TraceData 104 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 106 items :
    !0 #NS 0 A
    !1 #ELO #BI 54 !0 14
TraceData 106 ops :
    $0 infer (!1) 14 []


TraceData 107 items :
    !0 #SOME 5
TraceData 107 ops :
    $0 whnf (14) 14 [$1]
    $1 $0 whnf_core (14, !0) 14 []


TraceData 108 items :
    !0 #NS 0 A
    !1 #ELO #BI 54 !0 14
TraceData 108 ops :
    $0 infer (!1) 14 []


TraceData 109 items :
TraceData 109 ops :
    $0 whnf (14) 14 []


TraceData 110 items :
    !0 #ES 1
    !1 #US 1
    !2 #ES !1
TraceData 110 ops :
    $0 infer (!0) !2 []


TraceData 111 items :
    !0 #US 1
    !1 #ES !0
    !2 #SOME 5
TraceData 111 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 105 items :
    !0 #NS 0 A
    !1 #ELO #BI 54 !0 14
    !2 #ES 1
    !3 #EP #BD 0 !1 !2
    !4 #EP #BD 0 !1 !3
    !5 #US 1
    !6 #UIM 13 !5
    !7 #UIM 13 !6
    !8 #ES !7
TraceData 105 ops :
    $0 infer (!4) !8 []


TraceData 112 items :
    !0 #US 1
    !1 #UIM 13 !0
    !2 #UIM 13 !1
    !3 #ES !2
    !4 #SOME 5
    !5 #UM 13 !0
    !6 #UIM 13 !5
    !7 #ES !6
TraceData 112 ops :
    $0 whnf (!3) !7 [$1]
    $1 $0 whnf_core (!3, !4) !7 []


TraceData 113 items :
TraceData 113 ops :
    $0 infer (14) 34 []


TraceData 114 items :
TraceData 114 ops :
    $0 whnf (34) 34 []


TraceData 102 items :
    !0 #NS 0 A
    !1 #NS 0 R
    !2 #ES 1
    !3 #EP #BD 0 17 !2
    !4 #EP #BD 0 16 !3
    !5 #EP #BD !1 !4 14
    !6 #EP #BI !0 14 !5
    !7 #US 1
    !8 #UM 13 !7
    !9 #UIM 13 !8
    !10 #UIM !9 33
    !11 #UIM 33 !10
    !12 #ES !11
TraceData 102 ops :
    $0 infer (!6) !12 []


TraceData 115 items :
    !0 #US 1
    !1 #UM 13 !0
    !2 #UIM 13 !1
    !3 #UIM !2 33
    !4 #UIM 33 !3
    !5 #ES !4
    !6 #SOME 5
    !7 #UM !2 33
    !8 #UIM 33 !7
    !9 #ES !8
TraceData 115 ops :
    $0 whnf (!5) !9 [$1]
    $1 $0 whnf_core (!5, !6) !9 []


TraceData 117 items :
TraceData 117 ops :
    $0 infer (14) 34 []


TraceData 118 items :
    !0 #SOME 5
TraceData 118 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 120 items :
    !0 #NS 0 A
    !1 #ELO #BI 58 !0 14
TraceData 120 ops :
    $0 infer (!1) 14 []


TraceData 121 items :
    !0 #SOME 5
TraceData 121 ops :
    $0 whnf (14) 14 [$1]
    $1 $0 whnf_core (14, !0) 14 []


TraceData 122 items :
    !0 #NS 0 A
    !1 #ELO #BI 58 !0 14
TraceData 122 ops :
    $0 infer (!1) 14 []


TraceData 123 items :
TraceData 123 ops :
    $0 whnf (14) 14 []


TraceData 124 items :
    !0 #ES 1
    !1 #US 1
    !2 #ES !1
TraceData 124 ops :
    $0 infer (!0) !2 []


TraceData 125 items :
    !0 #US 1
    !1 #ES !0
    !2 #SOME 5
TraceData 125 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 119 items :
    !0 #NS 0 A
    !1 #ELO #BI 58 !0 14
    !2 #ES 1
    !3 #EP #BD 0 !1 !2
    !4 #EP #BD 0 !1 !3
    !5 #US 1
    !6 #UIM 13 !5
    !7 #UIM 13 !6
    !8 #ES !7
TraceData 119 ops :
    $0 infer (!4) !8 []


TraceData 126 items :
    !0 #US 1
    !1 #UIM 13 !0
    !2 #UIM 13 !1
    !3 #ES !2
    !4 #SOME 5
    !5 #UM 13 !0
    !6 #UIM 13 !5
    !7 #ES !6
TraceData 126 ops :
    $0 whnf (!3) !7 [$1]
    $1 $0 whnf_core (!3, !4) !7 []


TraceData 127 items :
    !0 #NS 0 A
    !1 #ELO #BI 58 !0 14
TraceData 127 ops :
    $0 infer (!1) 14 []


TraceData 128 items :
TraceData 128 ops :
    $0 whnf (14) 14 []


TraceData 129 items :
    !0 #NS 0 quot
    !1 #SEQ 13
    !2 #EC !0 13 
    !3 #NS 0 A
    !4 #ELO #BI 58 !3 14
    !5 #EA !2 !4
    !6 #NS 0 R
    !7 #ES 1
    !8 #EP #BD 0 !4 !7
    !9 #EP #BD 0 !4 !8
    !10 #ELO #BD 61 !6 !9
    !11 #EA !5 !10
    !12 #EP #BD 0 17 !7
    !13 #EP #BD 0 16 !12
    !14 #EP #BD !6 !13 14
    !15 #EP #BI !3 14 !14
TraceData 129 ops :
    $0 infer (!11) 14 [$1]
    $1 $0 infer_apps (!11) 14 [$2, $3, $6]
    $2 $1 infer (!2) !15 []
    $3 $1 check_type (!4, 14) 9 [$4, $5]
    $4 $3 infer (!4) 14 []
    $5 $3 check_def_eq (14, 14) 3 []
    $6 $1 check_type (!10, !9) 9 [$7, $8]
    $7 $6 infer (!10) !9 []
    $8 $6 check_def_eq (!9, !9) 3 []


TraceData 130 items :
TraceData 130 ops :
    $0 whnf (14) 14 []


TraceData 116 items :
    !0 #NS 0 A
    !1 #NS 0 R
    !2 #ES 1
    !3 #EP #BD 0 17 !2
    !4 #EP #BD 0 16 !3
    !5 #NS 0 quot
    !6 #SEQ 13
    !7 #EC !5 13 
    !8 #EA !7 38
    !9 #EA !8 17
    !10 #EP #BD 0 17 !9
    !11 #EP #BD !1 !4 !10
    !12 #EP #BI !0 14 !11
    !13 #US 1
    !14 #UM 13 !13
    !15 #UIM 13 !14
    !16 #UIM 13 13
    !17 #UIM !15 !16
    !18 #UIM 33 !17
    !19 #ES !18
TraceData 116 ops :
    $0 infer (!12) !19 []


TraceData 131 items :
    !0 #US 1
    !1 #UM 13 !0
    !2 #UIM 13 !1
    !3 #UIM 13 13
    !4 #UIM !2 !3
    !5 #UIM 33 !4
    !6 #ES !5
    !7 #SOME 5
TraceData 131 ops :
    $0 whnf (!6) !6 [$1]
    $1 $0 whnf_core (!6, !7) !6 []


TraceData 133 items :
TraceData 133 ops :
    $0 infer (14) 34 []


TraceData 134 items :
    !0 #SOME 5
TraceData 134 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 136 items :
    !0 #NS 0 A
    !1 #ELO #BI 63 !0 14
TraceData 136 ops :
    $0 infer (!1) 14 []


TraceData 137 items :
    !0 #SOME 5
TraceData 137 ops :
    $0 whnf (14) 14 [$1]
    $1 $0 whnf_core (14, !0) 14 []


TraceData 138 items :
    !0 #NS 0 A
    !1 #ELO #BI 63 !0 14
TraceData 138 ops :
    $0 infer (!1) 14 []


TraceData 139 items :
TraceData 139 ops :
    $0 whnf (14) 14 []


TraceData 140 items :
    !0 #ES 1
    !1 #US 1
    !2 #ES !1
TraceData 140 ops :
    $0 infer (!0) !2 []


TraceData 141 items :
    !0 #US 1
    !1 #ES !0
    !2 #SOME 5
TraceData 141 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 135 items :
    !0 #NS 0 A
    !1 #ELO #BI 63 !0 14
    !2 #ES 1
    !3 #EP #BD 0 !1 !2
    !4 #EP #BD 0 !1 !3
    !5 #US 1
    !6 #UIM 13 !5
    !7 #UIM 13 !6
    !8 #ES !7
TraceData 135 ops :
    $0 infer (!4) !8 []


TraceData 142 items :
    !0 #US 1
    !1 #UIM 13 !0
    !2 #UIM 13 !1
    !3 #ES !2
    !4 #SOME 5
    !5 #UM 13 !0
    !6 #UIM 13 !5
    !7 #ES !6
TraceData 142 ops :
    $0 whnf (!3) !7 [$1]
    $1 $0 whnf_core (!3, !4) !7 []


TraceData 144 items :
    !0 #NS 0 quot
    !1 #SEQ 13
    !2 #EC !0 13 
    !3 #NS 0 A
    !4 #ELO #BI 63 !3 14
    !5 #EA !2 !4
    !6 #NS 0 R
    !7 #ES 1
    !8 #EP #BD 0 !4 !7
    !9 #EP #BD 0 !4 !8
    !10 #ELO #BD 66 !6 !9
    !11 #EA !5 !10
    !12 #EP #BD 0 17 !7
    !13 #EP #BD 0 16 !12
    !14 #EP #BD !6 !13 14
    !15 #EP #BI !3 14 !14
TraceData 144 ops :
    $0 infer (!11) 14 [$1]
    $1 $0 infer_apps (!11) 14 [$2, $3, $6]
    $2 $1 infer (!2) !15 []
    $3 $1 check_type (!4, 14) 9 [$4, $5]
    $4 $3 infer (!4) 14 []
    $5 $3 check_def_eq (14, 14) 3 []
    $6 $1 check_type (!10, !9) 9 [$7, $8]
    $7 $6 infer (!10) !9 []
    $8 $6 check_def_eq (!9, !9) 3 []


TraceData 145 items :
TraceData 145 ops :
    $0 whnf (14) 14 []


TraceData 146 items :
    !0 #ES 1
    !1 #US 1
    !2 #ES !1
TraceData 146 ops :
    $0 infer (!0) !2 []


TraceData 147 items :
    !0 #US 1
    !1 #ES !0
TraceData 147 ops :
    $0 whnf (!1) !1 []


TraceData 143 items :
    !0 #NS 0 quot
    !1 #SEQ 13
    !2 #EC !0 13 
    !3 #NS 0 A
    !4 #ELO #BI 63 !3 14
    !5 #EA !2 !4
    !6 #NS 0 R
    !7 #ES 1
    !8 #EP #BD 0 !4 !7
    !9 #EP #BD 0 !4 !8
    !10 #ELO #BD 66 !6 !9
    !11 #EA !5 !10
    !12 #EP #BD 0 !11 !7
    !13 #US 1
    !14 #UIM 13 !13
    !15 #ES !14
TraceData 143 ops :
    $0 infer (!12) !15 []


TraceData 148 items :
    !0 #US 1
    !1 #UIM 13 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 !0
    !5 #ES !4
TraceData 148 ops :
    $0 whnf (!2) !5 [$1]
    $1 $0 whnf_core (!2, !3) !5 []


TraceData 150 items :
    !0 #NS 0 A
    !1 #ELO #BI 63 !0 14
TraceData 150 ops :
    $0 infer (!1) 14 []


TraceData 151 items :
TraceData 151 ops :
    $0 whnf (14) 14 []


TraceData 152 items :
    !0 #NS 0 B
    !1 #NS 0 quot
    !2 #SEQ 13
    !3 #EC !1 13 
    !4 #NS 0 A
    !5 #ELO #BI 63 !4 14
    !6 #EA !3 !5
    !7 #NS 0 R
    !8 #ES 1
    !9 #EP #BD 0 !5 !8
    !10 #EP #BD 0 !5 !9
    !11 #ELO #BD 66 !7 !10
    !12 #EA !6 !11
    !13 #EP #BD 0 !12 !8
    !14 #ELO #BI 68 !0 !13
    !15 #NS !1 mk
    !16 #EC !15 13 
    !17 #EA !16 !5
    !18 #EA !17 !11
    !19 #ELO #BD 69 15 !5
    !20 #EA !18 !19
    !21 #EA !14 !20
    !22 #EP #BD 0 17 !8
    !23 #EP #BD 0 16 !22
    !24 #EA !3 38
    !25 #EA !24 17
    !26 #EP #BD 0 17 !25
    !27 #EP #BD !7 !23 !26
    !28 #EP #BI !4 14 !27
TraceData 152 ops :
    $0 infer (!21) !8 [$1]
    $1 $0 infer_apps (!21) !8 [$2, $3]
    $2 $1 infer (!14) !13 []
    $3 $1 check_type (!20, !12) 9 [$4, $16]
    $4 $3 infer (!20) !12 [$5]
    $5 $4 infer_apps (!20) !12 [$6, $7, $10, $13]
    $6 $5 infer (!16) !28 []
    $7 $5 check_type (!5, 14) 9 [$8, $9]
    $8 $7 infer (!5) 14 []
    $9 $7 check_def_eq (14, 14) 3 []
    $10 $5 check_type (!11, !10) 9 [$11, $12]
    $11 $10 infer (!11) !10 []
    $12 $10 check_def_eq (!10, !10) 3 []
    $13 $5 check_type (!19, !5) 9 [$14, $15]
    $14 $13 infer (!19) !5 []
    $15 $13 check_def_eq (!5, !5) 3 []
    $16 $3 check_def_eq (!12, !12) 3 []


TraceData 153 items :
    !0 #ES 1
    !1 #SOME 5
TraceData 153 ops :
    $0 whnf (!0) 18 [$1]
    $1 $0 whnf_core (!0, !1) 18 []


TraceData 149 items :
    !0 #NS 0 A
    !1 #ELO #BI 63 !0 14
    !2 #NS 0 B
    !3 #NS 0 quot
    !4 #SEQ 13
    !5 #EC !3 13 
    !6 #EA !5 !1
    !7 #NS 0 R
    !8 #ES 1
    !9 #EP #BD 0 !1 !8
    !10 #EP #BD 0 !1 !9
    !11 #ELO #BD 66 !7 !10
    !12 #EA !6 !11
    !13 #EP #BD 0 !12 !8
    !14 #ELO #BI 68 !2 !13
    !15 #NS !3 mk
    !16 #EC !15 13 
    !17 #EA !16 !1
    !18 #EA !17 !11
    !19 #EA !18 16
    !20 #EA !14 !19
    !21 #EP #BD 15 !1 !20
    !22 #UIM 13 1
    !23 #ES !22
TraceData 149 ops :
    $0 infer (!21) !23 []


TraceData 154 items :
    !0 #UIM 13 1
    !1 #ES !0
    !2 #SOME 5
TraceData 154 ops :
    $0 whnf (!1) 18 [$1]
    $1 $0 whnf_core (!1, !2) 18 []


TraceData 155 items :
    !0 #NS 0 quot
    !1 #SEQ 13
    !2 #EC !0 13 
    !3 #NS 0 A
    !4 #ELO #BI 63 !3 14
    !5 #EA !2 !4
    !6 #NS 0 R
    !7 #ES 1
    !8 #EP #BD 0 !4 !7
    !9 #EP #BD 0 !4 !8
    !10 #ELO #BD 66 !6 !9
    !11 #EA !5 !10
TraceData 155 ops :
    $0 infer (!11) 14 []


TraceData 156 items :
TraceData 156 ops :
    $0 whnf (14) 14 []


TraceData 157 items :
    !0 #NS 0 B
    !1 #NS 0 quot
    !2 #SEQ 13
    !3 #EC !1 13 
    !4 #NS 0 A
    !5 #ELO #BI 63 !4 14
    !6 #EA !3 !5
    !7 #NS 0 R
    !8 #ES 1
    !9 #EP #BD 0 !5 !8
    !10 #EP #BD 0 !5 !9
    !11 #ELO #BD 66 !7 !10
    !12 #EA !6 !11
    !13 #EP #BD 0 !12 !8
    !14 #ELO #BI 68 !0 !13
    !15 #NS 0 q
    !16 #ELO #BD 71 !15 !12
    !17 #EA !14 !16
TraceData 157 ops :
    $0 infer (!17) !8 [$1]
    $1 $0 infer_apps (!17) !8 [$2, $3]
    $2 $1 infer (!14) !13 []
    $3 $1 check_type (!16, !12) 9 [$4, $5]
    $4 $3 infer (!16) !12 []
    $5 $3 check_def_eq (!12, !12) 3 []


TraceData 158 items :
    !0 #ES 1
TraceData 158 ops :
    $0 whnf (!0) 18 []


TraceData 132 items :
    !0 #NS 0 A
    !1 #NS 0 R
    !2 #ES 1
    !3 #EP #BD 0 17 !2
    !4 #EP #BD 0 16 !3
    !5 #NS 0 B
    !6 #NS 0 quot
    !7 #SEQ 13
    !8 #EC !6 13 
    !9 #EA !8 17
    !10 #EA !9 16
    !11 #EP #BD 0 !10 !2
    !12 #NS !6 mk
    !13 #EC !12 13 
    !14 #EA !13 47
    !15 #EA !14 38
    !16 #EA !15 16
    !17 #EA 17 !16
    !18 #EP #BD 15 38 !17
    !19 #NS 0 q
    !20 #EA !8 47
    !21 #EA !20 38
    !22 #EA 38 16
    !23 #EP #BD !19 !21 !22
    !24 #EP #BD 0 !18 !23
    !25 #EP #BI !5 !11 !24
    !26 #EP #BD !1 !4 !25
    !27 #EP #BI !0 14 !26
    !28 #US 1
    !29 #UM 13 !28
    !30 #UIM 13 !29
    !31 #UIM 13 1
    !32 #UIM 1 !31
    !33 #UIM !29 !32
    !34 #UIM !30 !33
    !35 #UIM 33 !34
    !36 #ES !35
TraceData 132 ops :
    $0 infer (!27) !36 []


TraceData 159 items :
    !0 #US 1
    !1 #UM 13 !0
    !2 #UIM 13 !1
    !3 #UIM 13 1
    !4 #UIM 1 !3
    !5 #UIM !1 !4
    !6 #UIM !2 !5
    !7 #UIM 33 !6
    !8 #ES !7
    !9 #SOME 5
TraceData 159 ops :
    $0 whnf (!8) 18 [$1]
    $1 $0 whnf_core (!8, !9) 18 []


TraceData 161 items :
TraceData 161 ops :
    $0 infer (14) 34 []


TraceData 162 items :
    !0 #SOME 5
TraceData 162 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 164 items :
    !0 #NS 0 A
    !1 #ELO #BI 72 !0 14
TraceData 164 ops :
    $0 infer (!1) 14 []


TraceData 165 items :
    !0 #SOME 5
TraceData 165 ops :
    $0 whnf (14) 14 [$1]
    $1 $0 whnf_core (14, !0) 14 []


TraceData 166 items :
    !0 #NS 0 A
    !1 #ELO #BI 72 !0 14
TraceData 166 ops :
    $0 infer (!1) 14 []


TraceData 167 items :
TraceData 167 ops :
    $0 whnf (14) 14 []


TraceData 168 items :
    !0 #ES 1
    !1 #US 1
    !2 #ES !1
TraceData 168 ops :
    $0 infer (!0) !2 []


TraceData 169 items :
    !0 #US 1
    !1 #ES !0
    !2 #SOME 5
TraceData 169 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 163 items :
    !0 #NS 0 A
    !1 #ELO #BI 72 !0 14
    !2 #ES 1
    !3 #EP #BD 0 !1 !2
    !4 #EP #BD 0 !1 !3
    !5 #US 1
    !6 #UIM 13 !5
    !7 #UIM 13 !6
    !8 #ES !7
TraceData 163 ops :
    $0 infer (!4) !8 []


TraceData 170 items :
    !0 #US 1
    !1 #UIM 13 !0
    !2 #UIM 13 !1
    !3 #ES !2
    !4 #SOME 5
    !5 #UM 13 !0
    !6 #UIM 13 !5
    !7 #ES !6
TraceData 170 ops :
    $0 whnf (!3) !7 [$1]
    $1 $0 whnf_core (!3, !4) !7 []


TraceData 171 items :
    !0 #NS 0 v
    !1 #UP !0
    !2 #ES !1
    !3 #US !1
    !4 #ES !3
TraceData 171 ops :
    $0 infer (!2) !4 []


TraceData 172 items :
    !0 #NS 0 v
    !1 #UP !0
    !2 #US !1
    !3 #ES !2
    !4 #SOME 5
TraceData 172 ops :
    $0 whnf (!3) !3 [$1]
    $1 $0 whnf_core (!3, !4) !3 []


TraceData 174 items :
    !0 #NS 0 A
    !1 #ELO #BI 72 !0 14
TraceData 174 ops :
    $0 infer (!1) 14 []


TraceData 175 items :
TraceData 175 ops :
    $0 whnf (14) 14 []


TraceData 176 items :
    !0 #NS 0 B
    !1 #NS 0 v
    !2 #UP !1
    !3 #ES !2
    !4 #ELO #BI 76 !0 !3
TraceData 176 ops :
    $0 infer (!4) !3 []


TraceData 177 items :
    !0 #NS 0 v
    !1 #UP !0
    !2 #ES !1
    !3 #SOME 5
TraceData 177 ops :
    $0 whnf (!2) !2 [$1]
    $1 $0 whnf_core (!2, !3) !2 []


TraceData 173 items :
    !0 #NS 0 A
    !1 #ELO #BI 72 !0 14
    !2 #NS 0 B
    !3 #NS 0 v
    !4 #UP !3
    !5 #ES !4
    !6 #ELO #BI 76 !2 !5
    !7 #EP #BD 0 !1 !6
    !8 #UIM 13 !4
    !9 #ES !8
TraceData 173 ops :
    $0 infer (!7) !9 []


TraceData 178 items :
    !0 #NS 0 v
    !1 #UP !0
    !2 #UIM 13 !1
    !3 #ES !2
    !4 #SOME 5
TraceData 178 ops :
    $0 whnf (!3) !3 [$1]
    $1 $0 whnf_core (!3, !4) !3 []


TraceData 180 items :
    !0 #NS 0 A
    !1 #ELO #BI 72 !0 14
TraceData 180 ops :
    $0 infer (!1) 14 []


TraceData 181 items :
TraceData 181 ops :
    $0 whnf (14) 14 []


TraceData 182 items :
    !0 #NS 0 A
    !1 #ELO #BI 72 !0 14
TraceData 182 ops :
    $0 infer (!1) 14 []


TraceData 183 items :
TraceData 183 ops :
    $0 whnf (14) 14 []


TraceData 184 items :
    !0 #NS 0 R
    !1 #NS 0 A
    !2 #ELO #BI 72 !1 14
    !3 #ES 1
    !4 #EP #BD 0 !2 !3
    !5 #EP #BD 0 !2 !4
    !6 #ELO #BD 75 !0 !5
    !7 #ELO #BD 79 15 !2
    !8 #EA !6 !7
    !9 #ELO #BD 80 69 !2
    !10 #EA !8 !9
TraceData 184 ops :
    $0 infer (!10) !3 [$1]
    $1 $0 infer_apps (!10) !3 [$2, $3, $6]
    $2 $1 infer (!6) !5 []
    $3 $1 check_type (!7, !2) 9 [$4, $5]
    $4 $3 infer (!7) !2 []
    $5 $3 check_def_eq (!2, !2) 3 []
    $6 $1 check_type (!9, !2) 9 [$7, $8]
    $7 $6 infer (!9) !2 []
    $8 $6 check_def_eq (!2, !2) 3 []


TraceData 185 items :
    !0 #ES 1
    !1 #SOME 5
TraceData 185 ops :
    $0 whnf (!0) 18 [$1]
    $1 $0 whnf_core (!0, !1) 18 []


TraceData 186 items :
    !0 #NS 0 v
    !1 #UP !0
    !2 #SEQ !1
    !3 #EC 11 !1 
    !4 #NS 0 B
    !5 #ES !1
    !6 #ELO #BI 76 !4 !5
    !7 #EA !3 !6
    !8 #NS 0 f
    !9 #NS 0 A
    !10 #ELO #BI 72 !9 14
    !11 #EP #BD 0 !10 !6
    !12 #ELO #BD 78 !8 !11
    !13 #ELO #BD 79 15 !10
    !14 #EA !12 !13
    !15 #EA !7 !14
    !16 #ELO #BD 80 69 !10
    !17 #EA !12 !16
    !18 #EA !15 !17
    !19 #EP #BI 12 !5 20
TraceData 186 ops :
    $0 infer (!18) 18 [$1]
    $1 $0 infer_apps (!18) 18 [$2, $3, $6, $14]
    $2 $1 infer (!3) !19 []
    $3 $1 check_type (!6, !5) 9 [$4, $5]
    $4 $3 infer (!6) !5 []
    $5 $3 check_def_eq (!5, !5) 3 []
    $6 $1 check_type (!14, !6) 9 [$7, $13]
    $7 $6 infer (!14) !6 [$8]
    $8 $7 infer_apps (!14) !6 [$9, $10]
    $9 $8 infer (!12) !11 []
    $10 $8 check_type (!13, !10) 9 [$11, $12]
    $11 $10 infer (!13) !10 []
    $12 $10 check_def_eq (!10, !10) 3 []
    $13 $6 check_def_eq (!6, !6) 3 []
    $14 $1 check_type (!17, !6) 9 [$15, $21]
    $15 $14 infer (!17) !6 [$16]
    $16 $15 infer_apps (!17) !6 [$17, $18]
    $17 $16 infer (!12) !11 []
    $18 $16 check_type (!16, !10) 9 [$19, $20]
    $19 $18 infer (!16) !10 []
    $20 $18 check_def_eq (!10, !10) 3 []
    $21 $14 check_def_eq (!6, !6) 3 []


TraceData 187 items :
    !0 #SOME 5
TraceData 187 ops :
    $0 whnf (18) 18 [$1]
    $1 $0 whnf_core (18, !0) 18 []


TraceData 179 items :
    !0 #NS 0 A
    !1 #ELO #BI 72 !0 14
    !2 #NS 0 R
    !3 #ES 1
    !4 #EP #BD 0 !1 !3
    !5 #EP #BD 0 !1 !4
    !6 #ELO #BD 75 !2 !5
    !7 #EA !6 17
    !8 #EA !7 16
    !9 #NS 0 v
    !10 #UP !9
    !11 #SEQ !10
    !12 #EC 11 !10 
    !13 #NS 0 B
    !14 #ES !10
    !15 #ELO #BI 76 !13 !14
    !16 #EA !12 !15
    !17 #NS 0 f
    !18 #EP #BD 0 !1 !15
    !19 #ELO #BD 78 !17 !18
    !20 #EA !19 38
    !21 #EA !16 !20
    !22 #EA !19 17
    !23 #EA !21 !22
    !24 #EP #BD 0 !8 !23
    !25 #EP #BD 69 !1 !24
    !26 #EP #BD 15 !1 !25
    !27 #UIM 1 1
    !28 #UIM 13 !27
    !29 #UIM 13 !28
    !30 #ES !29
TraceData 179 ops :
    $0 infer (!26) !30 []


TraceData 188 items :
    !0 #UIM 1 1
    !1 #UIM 13 !0
    !2 #UIM 13 !1
    !3 #ES !2
    !4 #SOME 5
TraceData 188 ops :
    $0 whnf (!3) 18 [$1]
    $1 $0 whnf_core (!3, !4) 18 []


TraceData 189 items :
    !0 #NS 0 quot
    !1 #SEQ 13
    !2 #EC !0 13 
    !3 #NS 0 A
    !4 #ELO #BI 72 !3 14
    !5 #EA !2 !4
    !6 #NS 0 R
    !7 #ES 1
    !8 #EP #BD 0 !4 !7
    !9 #EP #BD 0 !4 !8
    !10 #ELO #BD 75 !6 !9
    !11 #EA !5 !10
    !12 #EP #BD 0 17 !7
    !13 #EP #BD 0 16 !12
    !14 #EP #BD !6 !13 14
    !15 #EP #BI !3 14 !14
TraceData 189 ops :
    $0 infer (!11) 14 [$1]
    $1 $0 infer_apps (!11) 14 [$2, $3, $6]
    $2 $1 infer (!2) !15 []
    $3 $1 check_type (!4, 14) 9 [$4, $5]
    $4 $3 infer (!4) 14 []
    $5 $3 check_def_eq (14, 14) 3 []
    $6 $1 check_type (!10, !9) 9 [$7, $8]
    $7 $6 infer (!10) !9 []
    $8 $6 check_def_eq (!9, !9) 3 []


TraceData 190 items :
TraceData 190 ops :
    $0 whnf (14) 14 []


TraceData 191 items :
    !0 #NS 0 B
    !1 #NS 0 v
    !2 #UP !1
    !3 #ES !2
    !4 #ELO #BI 76 !0 !3
TraceData 191 ops :
    $0 infer (!4) !3 []


TraceData 192 items :
    !0 #NS 0 v
    !1 #UP !0
    !2 #ES !1
TraceData 192 ops :
    $0 whnf (!2) !2 []


TraceData 160 items :
    !0 #NS 0 A
    !1 #NS 0 R
    !2 #ES 1
    !3 #EP #BD 0 17 !2
    !4 #EP #BD 0 16 !3
    !5 #NS 0 B
    !6 #NS 0 v
    !7 #UP !6
    !8 #ES !7
    !9 #NS 0 f
    !10 #EP #BD 0 38 17
    !11 #EA 55 17
    !12 #EA !11 16
    !13 #SEQ !7
    !14 #EC 11 !7 
    !15 #EA !14 55
    !16 #EA 47 38
    !17 #EA !15 !16
    !18 #EA 47 17
    !19 #EA !17 !18
    !20 #EP #BD 0 !12 !19
    !21 #EP #BD 69 55 !20
    !22 #EP #BD 15 47 !21
    !23 #NS 0 quot
    !24 #SEQ 13
    !25 #EC !23 13 
    !26 #EA !25 55
    !27 #EA !26 47
    !28 #EP #BD 0 !27 47
    !29 #EP #BD 0 !22 !28
    !30 #EP #BD !9 !10 !29
    !31 #EP #BI !5 !8 !30
    !32 #EP #BD !1 !4 !31
    !33 #EP #BI !0 14 !32
    !34 #US 1
    !35 #UM 13 !34
    !36 #UIM 13 !35
    !37 #US !7
    !38 #UIM 13 !7
    !39 #UIM 1 !38
    !40 #UIM !38 !39
    !41 #UIM !37 !40
    !42 #UIM !36 !41
    !43 #UIM 33 !42
    !44 #ES !43
TraceData 160 ops :
    $0 infer (!33) !44 []


TraceData 193 items :
    !0 #US 1
    !1 #UM 13 !0
    !2 #UIM 13 !1
    !3 #NS 0 v
    !4 #UP !3
    !5 #US !4
    !6 #UIM 13 !4
    !7 #UIM 1 !6
    !8 #UIM !6 !7
    !9 #UIM !5 !8
    !10 #UIM !2 !9
    !11 #UIM 33 !10
    !12 #ES !11
    !13 #SOME 5
TraceData 193 ops :
    $0 whnf (!12) !12 [$1]
    $1 $0 whnf_core (!12, !13) !12 []


TraceData 195 items :
    !0 #US 33
    !1 #ES !0
TraceData 195 ops :
    $0 infer (34) !1 []


TraceData 196 items :
    !0 #US 33
    !1 #ES !0
    !2 #SOME 5
TraceData 196 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 197 items :
    !0 #US 33
    !1 #ES !0
TraceData 197 ops :
    $0 infer (34) !1 []


TraceData 198 items :
    !0 #US 33
    !1 #ES !0
TraceData 198 ops :
    $0 whnf (!1) !1 []


TraceData 194 items :
    !0 #US 33
    !1 #UIM !0 !0
    !2 #ES !1
TraceData 194 ops :
    $0 infer (35) !2 []


TraceData 199 items :
    !0 #US 33
    !1 #UIM !0 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 13
    !5 #US !4
    !6 #US !5
    !7 #ES !6
TraceData 199 ops :
    $0 whnf (!2) !7 [$1]
    $1 $0 whnf_core (!2, !3) !7 []


TraceData 201 items :
    !0 #US 33
    !1 #ES !0
TraceData 201 ops :
    $0 infer (34) !1 []


TraceData 202 items :
    !0 #US 33
    !1 #ES !0
    !2 #SOME 5
TraceData 202 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 204 items :
    !0 #ELO #BI 85 12 34
TraceData 204 ops :
    $0 infer (!0) 34 []


TraceData 205 items :
    !0 #SOME 5
TraceData 205 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 206 items :
    !0 #ELO #BI 85 12 34
TraceData 206 ops :
    $0 infer (!0) 34 []


TraceData 207 items :
TraceData 207 ops :
    $0 whnf (34) 34 []


TraceData 208 items :
    !0 #ELO #BI 85 12 34
TraceData 208 ops :
    $0 infer (!0) 34 []


TraceData 209 items :
TraceData 209 ops :
    $0 whnf (34) 34 []


TraceData 203 items :
    !0 #ELO #BI 85 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 203 ops :
    $0 infer (!2) !5 []


TraceData 210 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 13
    !5 #UM 13 !4
    !6 #US !5
    !7 #ES !6
TraceData 210 ops :
    $0 whnf (!2) !7 [$1]
    $1 $0 whnf_core (!2, !3) !7 []


TraceData 211 items :
    !0 #ELO #BI 85 12 34
    !1 #EA 41 !0
TraceData 211 ops :
    $0 infer (!1) 34 [$1]
    $1 $0 infer_apps (!1) 34 [$2, $3]
    $2 $1 infer (41) 35 []
    $3 $1 check_type (!0, 34) 9 [$4, $5]
    $4 $3 infer (!0) 34 []
    $5 $3 check_def_eq (34, 34) 3 []


TraceData 212 items :
TraceData 212 ops :
    $0 whnf (34) 34 []


TraceData 200 items :
    !0 #US 33
    !1 #UM 13 13
    !2 #UM 13 !1
    !3 #US !2
    !4 #UIM !3 33
    !5 #UIM !0 !4
    !6 #ES !5
TraceData 200 ops :
    $0 infer (44) !6 []


TraceData 213 items :
    !0 #US 33
    !1 #UM 13 13
    !2 #UM 13 !1
    !3 #US !2
    !4 #UIM !3 33
    !5 #UIM !0 !4
    !6 #ES !5
    !7 #SOME 5
    !8 #UM !2 13
    !9 #UM 33 !8
    !10 #US !9
    !11 #ES !10
TraceData 213 ops :
    $0 whnf (!6) !11 [$1]
    $1 $0 whnf_core (!6, !7) !11 []


TraceData 215 items :
    !0 #US 33
    !1 #ES !0
TraceData 215 ops :
    $0 infer (34) !1 []


TraceData 216 items :
    !0 #US 33
    !1 #ES !0
    !2 #SOME 5
TraceData 216 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 217 items :
    !0 #ELO #BI 89 12 34
    !1 #EA 41 !0
TraceData 217 ops :
    $0 infer (!1) 34 [$1]
    $1 $0 infer_apps (!1) 34 [$2, $3]
    $2 $1 infer (41) 35 []
    $3 $1 check_type (!0, 34) 9 [$4, $5]
    $4 $3 infer (!0) 34 []
    $5 $3 check_def_eq (34, 34) 3 []


TraceData 218 items :
    !0 #SOME 5
TraceData 218 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 219 items :
    !0 #ELO #BI 89 12 34
TraceData 219 ops :
    $0 infer (!0) 34 []


TraceData 220 items :
TraceData 220 ops :
    $0 whnf (34) 34 []


TraceData 221 items :
    !0 #ELO #BI 89 12 34
TraceData 221 ops :
    $0 infer (!0) 34 []


TraceData 222 items :
TraceData 222 ops :
    $0 whnf (34) 34 []


TraceData 223 items :
    !0 #ELO #BI 89 12 34
TraceData 223 ops :
    $0 infer (!0) 34 []


TraceData 224 items :
TraceData 224 ops :
    $0 whnf (34) 34 []


TraceData 214 items :
    !0 #US 33
    !1 #UIM 33 33
    !2 #UIM 33 !1
    !3 #UIM 33 !2
    !4 #UIM !0 !3
    !5 #ES !4
TraceData 214 ops :
    $0 infer (51) !5 []


TraceData 225 items :
    !0 #US 33
    !1 #UIM 33 33
    !2 #UIM 33 !1
    !3 #UIM 33 !2
    !4 #UIM !0 !3
    !5 #ES !4
    !6 #SOME 5
    !7 #UM 13 13
    !8 #UM 13 !7
    !9 #UM 13 !8
    !10 #UM 33 !9
    !11 #US !10
    !12 #ES !11
TraceData 225 ops :
    $0 whnf (!5) !12 [$1]
    $1 $0 whnf_core (!5, !6) !12 []


TraceData 227 items :
    !0 #US 33
    !1 #ES !0
TraceData 227 ops :
    $0 infer (34) !1 []


TraceData 228 items :
    !0 #US 33
    !1 #ES !0
TraceData 228 ops :
    $0 whnf (!1) !1 []


TraceData 229 items :
    !0 #ELO #BD 93 12 34
    !1 #EA 41 !0
TraceData 229 ops :
    $0 infer (!1) 34 [$1]
    $1 $0 infer_apps (!1) 34 [$2, $3]
    $2 $1 infer (41) 35 []
    $3 $1 check_type (!0, 34) 9 [$4, $5]
    $4 $3 infer (!0) 34 []
    $5 $3 check_def_eq (34, 34) 3 []


TraceData 230 items :
TraceData 230 ops :
    $0 whnf (34) 34 []


TraceData 232 items :
    !0 #ELO #BD 93 12 34
    !1 #EA 41 !0
TraceData 232 ops :
    $0 infer (!1) 34 []


TraceData 233 items :
TraceData 233 ops :
    $0 whnf (34) 34 []


TraceData 235 items :
    !0 #ELO #BD 93 12 34
TraceData 235 ops :
    $0 infer (!0) 34 []


TraceData 236 items :
TraceData 236 ops :
    $0 whnf (34) 34 []


TraceData 237 items :
    !0 #ELO #BD 93 12 34
TraceData 237 ops :
    $0 infer (!0) 34 []


TraceData 238 items :
TraceData 238 ops :
    $0 whnf (34) 34 []


TraceData 239 items :
    !0 #ELO #BD 93 12 34
TraceData 239 ops :
    $0 infer (!0) 34 []


TraceData 240 items :
TraceData 240 ops :
    $0 whnf (34) 34 []


TraceData 234 items :
    !0 #ELO #BD 93 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 234 ops :
    $0 infer (!2) !5 []


TraceData 242 items :
    !0 #ELO #BD 93 12 34
    !1 #EA 41 !0
TraceData 242 ops :
    $0 infer (!1) 34 []


TraceData 243 items :
TraceData 243 ops :
    $0 whnf (34) 34 []


TraceData 244 items :
    !0 #US 33
    !1 #ES !0
TraceData 244 ops :
    $0 infer (34) !1 []


TraceData 245 items :
    !0 #US 33
    !1 #ES !0
TraceData 245 ops :
    $0 whnf (!1) !1 []


TraceData 241 items :
    !0 #ELO #BD 93 12 34
    !1 #EA 41 !0
    !2 #EP #BD 45 !1 34
    !3 #US 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 241 ops :
    $0 infer (!2) !5 []


TraceData 246 items :
    !0 #US 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #US !1
    !4 #ES !3
TraceData 246 ops :
    $0 infer (!2) !4 []


TraceData 247 items :
    !0 #US 33
    !1 #UIM 33 !0
    !2 #US !1
    !3 #ES !2
    !4 #SOME 5
    !5 #UM 13 33
    !6 #US !5
    !7 #US !6
    !8 #ES !7
TraceData 247 ops :
    $0 whnf (!3) !8 [$1]
    $1 $0 whnf_core (!3, !4) !8 []


TraceData 248 items :
    !0 #ELO #BD 93 12 34
    !1 #EA 41 !0
TraceData 248 ops :
    $0 check_def_eq (!1, !1) 3 []


TraceData 250 items :
    !0 #US 33
    !1 #ES !0
TraceData 250 ops :
    $0 infer (34) !1 []


TraceData 251 items :
    !0 #US 33
    !1 #ES !0
    !2 #US !0
    !3 #ES !2
TraceData 251 ops :
    $0 infer (!1) !3 []


TraceData 252 items :
    !0 #US 33
    !1 #US !0
    !2 #ES !1
    !3 #SOME 5
TraceData 252 ops :
    $0 whnf (!2) !2 [$1]
    $1 $0 whnf_core (!2, !3) !2 []


TraceData 249 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #SOME 6
    !4 #UM 13 13
    !5 #UM 13 !4
    !6 #US !5
    !7 #ES !6
TraceData 249 ops :
    $0 check_def_eq (34, !2) 3 [$1]
    $1 $0 check_def_eq_core (34, !2) 3 [$2, $3]
    $2 $1 whnf_core (34, !3) 34 []
    $3 $1 whnf_core (!2, !3) !7 []


TraceData 253 items :
    !0 #ELO #BD 93 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 253 ops :
    $0 infer (!2) !5 []


TraceData 254 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 13
    !5 #UM 13 !4
    !6 #US !5
    !7 #ES !6
TraceData 254 ops :
    $0 whnf (!2) !7 [$1]
    $1 $0 whnf_core (!2, !3) !7 []


TraceData 255 items :
    !0 #ELO #BD 93 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #ELO #BD 99 37 !2
TraceData 255 ops :
    $0 infer (!3) !2 []


TraceData 257 items :
    !0 #ELO #BD 93 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 257 ops :
    $0 infer (!2) !5 []


TraceData 258 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #UM 13 13
    !4 #UM 13 !3
    !5 #US !4
    !6 #ES !5
TraceData 258 ops :
    $0 whnf (!2) !6 []


TraceData 259 items :
    !0 #ELO #BD 93 12 34
    !1 #EA 41 !0
    !2 #EP #BD 15 !0 !0
    !3 #EP #BD 15 !0 !2
    !4 #EL #BC 45 !1 !3
    !5 #EA 74 !0
    !6 #ELO #BD 100 37 !3
    !7 #EA !5 !6
    !8 #EA !4 !7
    !9 #UIM 33 33
    !10 #UIM 33 !9
    !11 #ES !10
    !12 #EP #BC 45 !1 !11
TraceData 259 ops :
    $0 infer (!8) !11 [$1]
    $1 $0 infer_apps (!8) !11 [$2, $3]
    $2 $1 infer (!4) !12 []
    $3 $1 check_type (!7, !1) 9 [$4, $13]
    $4 $3 infer (!7) !1 [$5]
    $5 $4 infer_apps (!7) !1 [$6, $7, $10]
    $6 $5 infer (74) 44 []
    $7 $5 check_type (!0, 34) 9 [$8, $9]
    $8 $7 infer (!0) 34 []
    $9 $7 check_def_eq (34, 34) 3 []
    $10 $5 check_type (!6, !3) 9 [$11, $12]
    $11 $10 infer (!6) !3 []
    $12 $10 check_def_eq (!3, !3) 3 []
    $13 $3 check_def_eq (!1, !1) 3 []


TraceData 260 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #UM 13 13
    !4 #UM 13 !3
    !5 #US !4
    !6 #ES !5
TraceData 260 ops :
    $0 whnf (!2) !6 []


TraceData 256 items :
    !0 #ELO #BD 93 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #EA 41 !0
    !4 #EL #BC 45 !3 !2
    !5 #EA 74 !0
    !6 #EA !5 16
    !7 #EA !4 !6
    !8 #EP #BD 37 !2 !7
    !9 #UM 13 13
    !10 #UM 13 !9
    !11 #US !10
    !12 #UIM !11 !11
    !13 #ES !12
TraceData 256 ops :
    $0 infer (!8) !13 []


TraceData 261 items :
    !0 #UM 13 13
    !1 #UM 13 !0
    !2 #US !1
    !3 #UIM !2 !2
    !4 #ES !3
    !5 #US !3
    !6 #ES !5
TraceData 261 ops :
    $0 infer (!4) !6 []


TraceData 262 items :
    !0 #UM 13 13
    !1 #UM 13 !0
    !2 #US !1
    !3 #UIM !2 !2
    !4 #US !3
    !5 #ES !4
    !6 #SOME 5
    !7 #UM !1 !1
    !8 #US !7
    !9 #US !8
    !10 #ES !9
TraceData 262 ops :
    $0 whnf (!5) !10 [$1]
    $1 $0 whnf_core (!5, !6) !10 []


TraceData 264 items :
    !0 #ELO #BD 93 12 34
    !1 #EA 41 !0
    !2 #EP #BD 15 !0 !0
    !3 #EP #BD 15 !0 !2
    !4 #EL #BC 45 !1 !3
    !5 #EA 74 !0
    !6 #ELO #BD 101 37 !3
    !7 #EA !5 !6
    !8 #EA !4 !7
    !9 #UIM 33 33
    !10 #UIM 33 !9
    !11 #ES !10
    !12 #EP #BC 45 !1 !11
TraceData 264 ops :
    $0 infer (!8) !11 [$1]
    $1 $0 infer_apps (!8) !11 [$2, $3]
    $2 $1 infer (!4) !12 []
    $3 $1 check_type (!7, !1) 9 [$4, $13]
    $4 $3 infer (!7) !1 [$5]
    $5 $4 infer_apps (!7) !1 [$6, $7, $10]
    $6 $5 infer (74) 44 []
    $7 $5 check_type (!0, 34) 9 [$8, $9]
    $8 $7 infer (!0) 34 []
    $9 $7 check_def_eq (34, 34) 3 []
    $10 $5 check_type (!6, !3) 9 [$11, $12]
    $11 $10 infer (!6) !3 []
    $12 $10 check_def_eq (!3, !3) 3 []
    $13 $3 check_def_eq (!1, !1) 3 []


TraceData 265 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #US !1
    !4 #ES !3
TraceData 265 ops :
    $0 infer (!2) !4 []


TraceData 266 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #US !1
    !3 #ES !2
    !4 #SOME 5
    !5 #UM 13 13
    !6 #UM 13 !5
    !7 #US !6
    !8 #US !7
    !9 #ES !8
TraceData 266 ops :
    $0 whnf (!3) !9 [$1]
    $1 $0 whnf_core (!3, !4) !9 []


TraceData 267 items :
    !0 #ELO #BD 93 12 34
TraceData 267 ops :
    $0 check_def_eq (!0, !0) 3 []


TraceData 263 items :
    !0 #ELO #BD 93 12 34
    !1 #EA 41 !0
    !2 #EP #BD 15 !0 !0
    !3 #EP #BD 15 !0 !2
    !4 #EL #BC 45 !1 !3
    !5 #EA 74 !0
    !6 #ELO #BD 101 37 !3
    !7 #EA !5 !6
    !8 #EA !4 !7
    !9 #SOME 6
TraceData 263 ops :
    $0 check_def_eq (!8, !3) 3 [$1]
    $1 $0 check_def_eq_core (!8, !3) 3 [$2, $4]
    $2 $1 whnf_core (!8, !9) !3 [$3]
    $3 $2 whnf_core (!3, !9) !3 []
    $4 $1 whnf_core (!3, !9) !3 []


TraceData 231 items :
    !0 #ELO #BD 93 12 34
    !1 #EA 53 !0
    !2 #EA 41 !0
    !3 #EP #BD 15 !0 !0
    !4 #EP #BD 15 !0 !3
    !5 #EL #BC 45 !2 !4
    !6 #EA !1 !5
    !7 #EL #BD 37 !4 16
    !8 #EA !6 !7
    !9 #ELO #BC 94 45 !2
    !10 #EA !8 !9
    !11 #NS 0 C
    !12 #EP #BD 45 46 34
    !13 #NS 0 h
    !14 #EA 74 38
    !15 #EA !14 16
    !16 #EA 17 !15
    !17 #EP #BD 37 49 !16
    !18 #NS 0 x
    !19 #EA 41 38
    !20 #EA 38 16
    !21 #EP #BD !18 !19 !20
    !22 #EP #BD !13 !17 !21
    !23 #EP #BI !11 !12 !22
    !24 #EP #BD 12 34 !23
    !25 #EP #BD 45 !2 34
    !26 #UIM 33 33
    !27 #UIM 33 !26
    !28 #ES !27
    !29 #EP #BC 45 !2 !28
    !30 #SOME 6
    !31 #EA 74 !0
    !32 #EA !31 16
    !33 #EA !5 !32
    !34 #EP #BD 37 !4 !33
    !35 #EP #BD 37 !4 !4
    !36 #EA !5 !9
TraceData 231 ops :
    $0 infer (!10) !36 [$1]
    $1 $0 infer_apps (!10) !36 [$2, $3, $6, $12, $18]
    $2 $1 infer (53) !24 []
    $3 $1 check_type (!0, 34) 9 [$4, $5]
    $4 $3 infer (!0) 34 []
    $5 $3 check_def_eq (34, 34) 3 []
    $6 $1 check_type (!5, !25) 9 [$7, $8]
    $7 $6 infer (!5) !29 []
    $8 $6 check_def_eq (!25, !29) 3 [$9]
    $9 $8 check_def_eq_core (!25, !29) 3 [$10, $11]
    $10 $9 whnf_core (!25, !30) !25 []
    $11 $9 whnf_core (!29, !30) !29 []
    $12 $1 check_type (!7, !34) 9 [$13, $14]
    $13 $12 infer (!7) !35 []
    $14 $12 check_def_eq (!34, !35) 3 [$15]
    $15 $14 check_def_eq_core (!34, !35) 3 [$16, $17]
    $16 $15 whnf_core (!34, !30) !34 []
    $17 $15 whnf_core (!35, !30) !35 []
    $18 $1 check_type (!9, !2) 9 [$19, $20]
    $19 $18 infer (!9) !2 []
    $20 $18 check_def_eq (!2, !2) 3 []


TraceData 268 items :
    !0 #US 33
    !1 #UIM 33 33
    !2 #UIM 33 !1
    !3 #UIM 33 !2
    !4 #UIM !0 !3
    !5 #ES !4
TraceData 268 ops :
    $0 infer (51) !5 []


TraceData 269 items :
    !0 #US 33
    !1 #UIM 33 33
    !2 #UIM 33 !1
    !3 #UIM 33 !2
    !4 #UIM !0 !3
    !5 #ES !4
    !6 #US !4
    !7 #ES !6
TraceData 269 ops :
    $0 infer (!5) !7 []


TraceData 270 items :
    !0 #US 33
    !1 #UIM 33 33
    !2 #UIM 33 !1
    !3 #UIM 33 !2
    !4 #UIM !0 !3
    !5 #US !4
    !6 #ES !5
    !7 #SOME 5
    !8 #UM 13 13
    !9 #UM 13 !8
    !10 #UM 13 !9
    !11 #UM 33 !10
    !12 #US !11
    !13 #US !12
    !14 #ES !13
TraceData 270 ops :
    $0 whnf (!6) !14 [$1]
    $1 $0 whnf_core (!6, !7) !14 []


TraceData 271 items :
TraceData 271 ops :
    $0 check_def_eq (34, 34) 3 []


TraceData 274 items :
    !0 #ELO #BD 102 12 34
TraceData 274 ops :
    $0 infer (!0) 34 []


TraceData 275 items :
TraceData 275 ops :
    $0 whnf (34) 34 []


TraceData 276 items :
    !0 #ELO #BD 102 12 34
TraceData 276 ops :
    $0 infer (!0) 34 []


TraceData 277 items :
TraceData 277 ops :
    $0 whnf (34) 34 []


TraceData 278 items :
    !0 #ELO #BD 102 12 34
TraceData 278 ops :
    $0 infer (!0) 34 []


TraceData 279 items :
TraceData 279 ops :
    $0 whnf (34) 34 []


TraceData 273 items :
    !0 #ELO #BD 102 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 273 ops :
    $0 infer (!2) !5 []


TraceData 280 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #US !1
    !4 #ES !3
TraceData 280 ops :
    $0 infer (!2) !4 []


TraceData 281 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #US !1
    !3 #ES !2
    !4 #UM 13 13
    !5 #UM 13 !4
    !6 #US !5
    !7 #US !6
    !8 #ES !7
TraceData 281 ops :
    $0 whnf (!3) !8 []


TraceData 282 items :
    !0 #ELO #BD 102 12 34
TraceData 282 ops :
    $0 check_def_eq (!0, !0) 3 []


TraceData 272 items :
    !0 #ELO #BD 102 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #EA 41 !0
    !4 #EL #BC 45 !3 !2
    !5 #ELO #BC 103 45 !3
    !6 #EA !4 !5
    !7 #SOME 6
TraceData 272 ops :
    $0 check_def_eq (!2, !6) 3 [$1]
    $1 $0 check_def_eq_core (!2, !6) 3 [$2, $3]
    $2 $1 whnf_core (!2, !7) !2 []
    $3 $1 whnf_core (!6, !7) !2 [$4]
    $4 $3 whnf_core (!2, !7) !2 []


TraceData 226 items :
    !0 #EA 58 16
    !1 #EP #BC 45 46 !0
    !2 #EP #BD 12 34 !1
    !3 #SOME 6
TraceData 226 ops :
    $0 check_type (64, 51) 9 [$1, $2]
    $1 $0 infer (64) !2 []
    $2 $0 check_def_eq (51, !2) 3 [$3]
    $3 $2 check_def_eq_core (51, !2) 3 [$4, $5]
    $4 $3 whnf_core (51, !3) 51 []
    $5 $3 whnf_core (!2, !3) !2 []


TraceData 284 items :
    !0 #US 33
    !1 #ES !0
TraceData 284 ops :
    $0 infer (34) !1 []


TraceData 285 items :
    !0 #US 33
    !1 #ES !0
    !2 #SOME 5
TraceData 285 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 286 items :
    !0 #US 33
    !1 #ES !0
TraceData 286 ops :
    $0 infer (34) !1 []


TraceData 287 items :
    !0 #US 33
    !1 #ES !0
TraceData 287 ops :
    $0 whnf (!1) !1 []


TraceData 283 items :
    !0 #US 33
    !1 #UIM !0 !0
    !2 #ES !1
TraceData 283 ops :
    $0 infer (35) !2 []


TraceData 288 items :
    !0 #US 33
    !1 #UIM !0 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 13
    !5 #US !4
    !6 #US !5
    !7 #ES !6
TraceData 288 ops :
    $0 whnf (!2) !7 [$1]
    $1 $0 whnf_core (!2, !3) !7 []


TraceData 290 items :
    !0 #US 33
    !1 #ES !0
TraceData 290 ops :
    $0 infer (34) !1 []


TraceData 291 items :
    !0 #US 33
    !1 #ES !0
    !2 #SOME 5
TraceData 291 ops :
    $0 whnf (!1) !1 [$1]
    $1 $0 whnf_core (!1, !2) !1 []


TraceData 293 items :
    !0 #ELO #BI 107 12 34
TraceData 293 ops :
    $0 infer (!0) 34 []


TraceData 294 items :
    !0 #SOME 5
TraceData 294 ops :
    $0 whnf (34) 34 [$1]
    $1 $0 whnf_core (34, !0) 34 []


TraceData 295 items :
    !0 #ELO #BI 107 12 34
TraceData 295 ops :
    $0 infer (!0) 34 []


TraceData 296 items :
TraceData 296 ops :
    $0 whnf (34) 34 []


TraceData 297 items :
    !0 #ELO #BI 107 12 34
TraceData 297 ops :
    $0 infer (!0) 34 []


TraceData 298 items :
TraceData 298 ops :
    $0 whnf (34) 34 []


TraceData 292 items :
    !0 #ELO #BI 107 12 34
    !1 #EP #BD 15 !0 !0
    !2 #EP #BD 15 !0 !1
    !3 #UIM 33 33
    !4 #UIM 33 !3
    !5 #ES !4
TraceData 292 ops :
    $0 infer (!2) !5 []


TraceData 299 items :
    !0 #UIM 33 33
    !1 #UIM 33 !0
    !2 #ES !1
    !3 #SOME 5
    !4 #UM 13 13
    !5 #UM 13 !4
    !6 #US !5
    !7 #ES !6
TraceData 299 ops :
    $0 whnf (!2) !7 [$1]
    $1 $0 whnf_core (!2, !3) !7 []


TraceData 301 items :
    !0 #ELO #BI 107 12 34
TraceData 301 ops :
    $0 infer (!0) 34 []


TraceData 302 items :
TraceData 302 ops :
    $0 whnf (34) 34 []


TraceData 303 items :
    !0 #ELO #BI 107 12 34
TraceData 303 ops :
    $0 infer (!0) 34 []


TraceData 304 items :
TraceData 304 ops :
    $0 whnf (34) 34 []


TraceData 305 items :
    !0 #ELO #BI 107 12 34
TraceData 305 ops :
    $0 infer (!0) 34 []


TraceData 306 items :
TraceData 306 ops :
    $0 whnf (34) 34 []


TraceData 307 items :
    !0 #ELO #BI 107 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 110 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #ELO #BD 111 15 !0
    !10 #EA !8 !9
    !11 #ELO #BD 112 69 !0
    !12 #EA !10 !11
    !13 #EA !8 !12
    !14 #ELO #BD 113 45 !0
    !15 #EA !13 !14
    !16 #EA !1 !15
    !17 #EA !8 !11
    !18 #EA !17 !14
    !19 #EA !10 !18
    !20 #EA !16 !19
    !21 #EP #BI 12 34 20
    !22 #EA 41 !0
TraceData 307 ops :
    $0 infer (!20) 18 [$1]
    $1 $0 infer_apps (!20) 18 [$2, $3, $6, $45]
    $2 $1 infer (70) !21 []
    $3 $1 check_type (!0, 34) 9 [$4, $5]
    $4 $3 infer (!0) 34 []
    $5 $3 check_def_eq (34, 34) 3 []
    $6 $1 check_type (!15, !0) 9 [$7, $44]
    $7 $6 infer (!15) !0 [$8]
    $8 $7 infer_apps (!15) !0 [$9, $10, $13, $24, $41]
    $9 $8 infer (72) 51 []
    $10 $8 check_type (!0, 34) 9 [$11, $12]
    $11 $10 infer (!0) 34 []
    $12 $10 check_def_eq (34, 34) 3 []
    $13 $8 check_type (!7, !22) 9 [$14, $23]
    $14 $13 infer (!7) !22 [$15]
    $15 $14 infer_apps (!7) !22 [$16, $17, $20]
    $16 $15 infer (74) 44 []
    $17 $15 check_type (!0, 34) 9 [$18, $19]
    $18 $17 infer (!0) 34 []
    $19 $17 check_def_eq (34, 34) 3 []
    $20 $15 check_type (!6, !5) 9 [$21, $22]
    $21 $20 infer (!6) !5 []
    $22 $20 check_def_eq (!5, !5) 3 []
    $23 $13 check_def_eq (!22, !22) 3 []
    $24 $8 check_type (!12, !0) 9 [$25, $40]
    $25 $24 infer (!12) !0 [$26]
    $26 $25 infer_apps (!12) !0 [$27, $28, $31, $34, $37]
    $27 $26 infer (72) 51 []
    $28 $26 check_type (!0, 34) 9 [$29, $30]
    $29 $28 infer (!0) 34 []
    $30 $28 check_def_eq (34, 34) 3 []
    $31 $26 check_type (!7, !22) 9 [$32, $33]
    $32 $31 infer (!7) !22 []
    $33 $31 check_def_eq (!22, !22) 3 []
    $34 $26 check_type (!9, !0) 9 [$35, $36]
    $35 $34 infer (!9) !0 []
    $36 $34 check_def_eq (!0, !0) 3 []
    $37 $26 check_type (!11, !0) 9 [$38, $39]
    $38 $37 infer (!11) !0 []
    $39 $37 check_def_eq (!0, !0) 3 []
    $40 $24 check_def_eq (!0, !0) 3 []
    $41 $8 check_type (!14, !0) 9 [$42, $43]
    $42 $41 infer (!14) !0 []
    $43 $41 check_def_eq (!0, !0) 3 []
    $44 $6 check_def_eq (!0, !0) 3 []
    $45 $1 check_type (!19, !0) 9 [$46, $75]
    $46 $45 infer (!19) !0 [$47]
    $47 $46 infer_apps (!19) !0 [$48, $49, $52, $55, $58]
    $48 $47 infer (72) 51 []
    $49 $47 check_type (!0, 34) 9 [$50, $51]
    $50 $49 infer (!0) 34 []
    $51 $49 check_def_eq (34, 34) 3 []
    $52 $47 check_type (!7, !22) 9 [$53, $54]
    $53 $52 infer (!7) !22 []
    $54 $52 check_def_eq (!22, !22) 3 []
    $55 $47 check_type (!9, !0) 9 [$56, $57]
    $56 $55 infer (!9) !0 []
    $57 $55 check_def_eq (!0, !0) 3 []
    $58 $47 check_type (!18, !0) 9 [$59, $74]
    $59 $58 infer (!18) !0 [$60]
    $60 $59 infer_apps (!18) !0 [$61, $62, $65, $68, $71]
    $61 $60 infer (72) 51 []
    $62 $60 check_type (!0, 34) 9 [$63, $64]
    $63 $62 infer (!0) 34 []
    $64 $62 check_def_eq (34, 34) 3 []
    $65 $60 check_type (!7, !22) 9 [$66, $67]
    $66 $65 infer (!7) !22 []
    $67 $65 check_def_eq (!22, !22) 3 []
    $68 $60 check_type (!11, !0) 9 [$69, $70]
    $69 $68 infer (!11) !0 []
    $70 $68 check_def_eq (!0, !0) 3 []
    $71 $60 check_type (!14, !0) 9 [$72, $73]
    $72 $71 infer (!14) !0 []
    $73 $71 check_def_eq (!0, !0) 3 []
    $74 $58 check_def_eq (!0, !0) 3 []
    $75 $45 check_def_eq (!0, !0) 3 []


TraceData 308 items :
    !0 #SOME 5
TraceData 308 ops :
    $0 whnf (18) 18 [$1]
    $1 $0 whnf_core (18, !0) 18 []


TraceData 300 items :
    !0 #ELO #BI 107 12 34
    !1 #EA 70 !0
    !2 #EA 72 !0
    !3 #EA 74 !0
    !4 #EP #BD 15 !0 !0
    !5 #EP #BD 15 !0 !4
    !6 #ELO #BD 110 37 !5
    !7 #EA !3 !6
    !8 #EA !2 !7
    !9 #EA !8 38
    !10 #EA !9 17
    !11 #EA !8 !10
    !12 #EA !11 16
    !13 #EA !1 !12
    !14 #EA !8 17
    !15 #EA !14 16
    !16 #EA !9 !15
    !17 #EA !13 !16
    !18 #EP #BD 45 !0 !17
    !19 #EP #BD 69 !0 !18
    !20 #EP #BD 15 !0 !19
    !21 #UIM 33 1
    !22 #UIM 33 !21
    !23 #UIM 33 !22
    !24 #ES !23
TraceData 300 ops :
    $0 infer (!20) !24 []


TraceData 309 items :
    !0 #UIM 33 1
    !1 #UIM 33 !0
    !2 #UIM 33 !1
    !3 #ES !2
    !4 #SOME 5
TraceData 309 ops :
    $0 whnf (!3) 18 [$1]
    $1 $0 whnf_core (!3, !4) 18 []


TraceData 310 items :
    !0 #ELO #BI 107 12 34
    !1 #EA 90 !0
TraceData 310 ops :
    $0 infer (!1) 34 [$1]
    $1 $0 infer_apps (!1) 34 [$2, $3]
    $2 $1 infer (90) 35 []
    $3 $1 check_type (!0, 34) 9 [$4, $5]
    $4 $3 infer (!0) 34 []
    $5 $3 check_def_eq (34, 34) 3 []


TraceData 311 items :
TraceData 311 ops :
    $0 whnf (34) 34 []


TraceData 289 items :
    !0 #US 33
    !1 #UM 13 13
    !2 #UM 13 !1
    !3 #US !2
    !4 #UIM 1 33
    !5 #UIM !3 !4
    !6 #UIM !0 !5
    !7 #ES !6
TraceData 289 ops :
    $0 infer (94) !7 []


TraceData 312 items :
    !0 #US 33
    !1 #UM 13 13
    !2 #UM 13 !1
    !3 #US !2
    !4 #UIM 1 33
    !5 #UIM !3 !4
    !6 #UIM !0 !5
    !7 #ES !6
    !8 #SOME 5
    !9 #UM !2 13
    !10 #UM 33 !9
    !11 #US !10
    !12 #ES !11
TraceData 312 ops :
    $0 whnf (!7) !12 [$1]
    $1 $0 whnf_core (!7, !8) !12 []



No items to pretty print

0 Anon
1 Zero
2 #NONE
3 #SSEQ
4 #SSNEQ
5 #FLAGT
6 #FLAGF
7 #TT
8 #FF
9 #UNIT
10 #NS 0 u
11 #NS 0 eq
12 #NS 0 α
13 #UP 10
14 #ES 13
15 #NS 0 a
16 #EV 0
17 #EV 1
18 #ES 1
19 #EP #BD 15 17 18
20 #EP #BD 15 16 19
21 #EP #BI 12 14 20
22 #NS 11 refl
23 #EC 11 13 
24 #EA 23 17
25 #EA 24 16
26 #EA 25 16
27 #EP #BD 15 16 26
28 #EP #BI 12 14 27
29 #NS 0 linear_ordered_semiring
30 #NS 29 le_of_add_le_add_left
31 #NS 0 has_add
32 #NS 31 add
33 #US 13
34 #ES 33
35 #EP #BD 12 34 34
36 #NS 31 mk
37 #NS 0 add
38 #EV 2
39 #EP #BD 15 17 38
40 #EP #BD 15 16 39
41 #EC 31 13 
42 #EA 41 17
43 #EP #BD 37 40 42
44 #EP #BI 12 34 43
45 #NS 0 c
46 #EA 41 16
47 #EV 3
48 #EP #BD 15 38 47
49 #EP #BD 15 17 48
50 #EP #BC 45 46 49
51 #EP #BI 12 34 50
52 #NS 31 rec
53 #EC 52 33 13 
54 #EA 53 17
55 #EV 4
56 #EP #BD 15 47 55
57 #EP #BD 15 38 56
58 #EL #BC 45 42 57
59 #EA 54 58
60 #EL #BD 37 49 16
61 #EA 59 60
62 #EA 61 16
63 #EL #BC 45 46 62
64 #EL #BD 12 34 63
65 #NS 0 add_semigroup
66 #NS 65 to_has_add
67 #NS 65 mk
68 #NS 0 add_assoc
69 #NS 0 b
70 #EC 11 33 
71 #EA 70 55
72 #EC 32 13 
73 #EA 72 55
74 #EC 36 13 
75 #EA 74 55
76 #EA 75 47
77 #EA 73 76
78 #EA 77 38
79 #EA 78 17
80 #EA 77 79
81 #EA 80 16
82 #EA 71 81
83 #EA 77 17
84 #EA 83 16
85 #EA 78 84
86 #EA 82 85
87 #EP #BD 45 47 86
88 #EP #BD 69 38 87
89 #EP #BD 15 17 88
90 #EC 65 13 
91 #EA 90 38
92 #EP #BD 68 89 91
93 #EP #BD 37 40 92
94 #EP #BI 12 34 93
95 #NS 65 add
96 #EA 90 16
97 #EP #BC 45 96 49
98 #EP #BI 12 34 97
99 #NS 65 rec
100 #EC 99 33 13 
101 #EA 100 17
102 #EA 90 17
103 #EL #BC 45 102 57
104 #EA 101 103
105 #EV 5
106 #EA 70 105
107 #EA 72 105
108 #EA 74 105
109 #EA 108 47
110 #EA 107 109
111 #EA 110 38
112 #EA 111 17
113 #EA 110 112
114 #EA 113 16
115 #EA 106 114
116 #EA 110 17
117 #EA 116 16
118 #EA 111 117
119 #EA 115 118
120 #EP #BD 45 55 119
121 #EP #BD 69 47 120
122 #EP #BD 15 38 121
123 #EL #BD 68 122 17
124 #EL #BD 37 49 123
125 #EA 104 124
126 #EA 125 16
127 #EL #BC 45 96 126
128 #EL #BD 12 34 127




### Finished checking 14 items in 27.053654ms; to the best of our knowledge, all terms were well-typed! ###

