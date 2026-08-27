# NANDA Triad Worksheet

task_id: w1-dafsa-typed-view-m3-v1-all-pass
domain: general
query: May the split M3 structure be accepted only when every required route receipt passes?

## triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| s1 | M3 structural acceptance | requires | Route A PASS | receipt SHA 7add9d3dd5645ecc9bf1ba9c37f3b8a07835c160296869e19a0387f798137ee5 | 1.0 | claim boundary | route receipt | closure | all-routes |
| s2 | M3 structural acceptance | requires | Route B PASS | receipt SHA f30a6bf5d7af71f95bf6fee7888b8e2f797ed4b6e175317a6eb2c6e255e9f3bb | 1.0 | claim boundary | route receipt | closure | all-routes |
| s3 | M3 structural acceptance | requires | Route C PASS | receipt SHA 46e2d136b622b519d4567b5f439a9cf5d733e7527d23769d867240d790c87f43 | 1.0 | claim boundary | route receipt | closure | all-routes |
| s4 | M3 structural acceptance | requires | no missing WATCH or VETO branch | split acceptance requires three exact PASS receipts | 1.0 | claim boundary | route closure | closure | all-routes |

## candidate_triads

| id | subject | relation | object | evidence | confidence | subject_role | object_role | route | group |
|----|---------|----------|--------|----------|------------|--------------|-------------|-------|-------|
| c1 | M3 structural acceptance | requires | Route A PASS | exact receipt 7add9d3dd5645ecc9bf1ba9c37f3b8a07835c160296869e19a0387f798137ee5 is PASS | 1.0 | claim boundary | route receipt | closure | all-routes |
| c2 | M3 structural acceptance | requires | Route B PASS | exact receipt f30a6bf5d7af71f95bf6fee7888b8e2f797ed4b6e175317a6eb2c6e255e9f3bb is PASS | 1.0 | claim boundary | route receipt | closure | all-routes |
| c3 | M3 structural acceptance | requires | Route C PASS | exact receipt 46e2d136b622b519d4567b5f439a9cf5d733e7527d23769d867240d790c87f43 is PASS | 1.0 | claim boundary | route receipt | closure | all-routes |
| c4 | M3 structural acceptance | requires | no missing WATCH or VETO branch | no partial structural promotion is permitted | 1.0 | claim boundary | route closure | closure | all-routes |

## notes

- The aggregate packet remains size-only WATCH.
- This claim gate is valid only with all three branch receipt identities.
