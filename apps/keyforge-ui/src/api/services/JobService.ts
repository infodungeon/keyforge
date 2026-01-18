// apps/keyforge-ui/src/api/services/JobService.ts

import { BaseService } from "./BaseService";
import { RegisterJobRequest, JobStatusUpdate } from "../../types";

export class JobService extends BaseService {
    async dispatchJob(
        hiveUrl: string,
        hiveSecret: string,
        request: RegisterJobRequest,
    ): Promise<string> {
        const res = await fetch(`${hiveUrl}/jobs`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                "X-Keyforge-Secret": hiveSecret,
            },
            body: JSON.stringify(request),
        });
        if (!res.ok) {
            const text = await res.text();
            throw new Error(`Dispatch failed (${res.status}): ${text}`);
        }
        const data = await res.json();
        return data.job_id;
    }

    async pollHiveStatus(
        hiveUrl: string,
        hiveSecret: string,
        jobId: string,
    ): Promise<JobStatusUpdate> {
        const res = await fetch(`${hiveUrl}/jobs/${jobId}/status`, {
            headers: { "X-Keyforge-Secret": hiveSecret },
        });
        if (!res.ok) throw new Error("Status poll failed");
        return res.json();
    }

    async submitUserLayout(
        hiveUrl: string,
        _hiveSecret: string,
        name: string,
        layout: string,
        author: string,
    ): Promise<string> {
        const res = await fetch(`${hiveUrl}/submissions`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name, layout, author }),
        });
        if (!res.ok) throw new Error("Submission failed");
        return "Submitted";
    }

    async checkHiveHealth(hiveUrl: string): Promise<string> {
        const res = await fetch(`${hiveUrl}/health`);
        if (!res.ok) throw new Error("Hive unhealthy");
        return "OK";
    }
}
