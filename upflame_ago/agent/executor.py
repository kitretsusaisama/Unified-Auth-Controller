from typing import Dict, Any, Optional
import subprocess
import sys
import io
import shlex
import os

class ToolExecutor:
    def __init__(self):
        self.tools = {
            "python": self.execute_python,
            "shell": self.execute_shell,
            "search": self.execute_search
        }

    def execute(self, tool_name: str, params: Dict[str, Any]) -> Dict[str, Any]:
        if tool_name not in self.tools:
            return {"status": "error", "message": f"Tool {tool_name} not found"}

        try:
            result = self.tools[tool_name](params)
            return {"status": "success", "result": result}
        except Exception as e:
            return {"status": "error", "message": str(e)}

    def execute_python(self, params: Dict[str, Any]) -> str:
        # sourcery skip: no-exec
        if os.getenv("ALLOW_UNSAFE_EXECUTION", "false").lower() != "true":
            return "Error: Python execution is disabled. Set ALLOW_UNSAFE_EXECUTION=true to enable."

        code = params.get("code", "")
        buffer = io.StringIO()
        sys.stdout = buffer
        try:
            # WARNING: This is unsafe! Only run in sandboxed environments.
            exec(code, {"__name__": "__main__"})
            output = buffer.getvalue()
        except Exception as e:
            output = str(e)
        finally:
            sys.stdout = sys.__stdout__
        return output

    def execute_shell(self, params: Dict[str, Any]) -> str:
        if os.getenv("ALLOW_UNSAFE_EXECUTION", "false").lower() != "true":
            return "Error: Shell execution is disabled. Set ALLOW_UNSAFE_EXECUTION=true to enable."

        command = params.get("command", "")
        try:
            # Use shlex to split command and shell=False for security
            args = shlex.split(command)
            result = subprocess.run(args, shell=False, capture_output=True, text=True, timeout=10)
            return result.stdout + result.stderr
        except Exception as e:
            return str(e)

    def execute_search(self, params: Dict[str, Any]) -> str:
        query = params.get("query", "")
        return f"Results for {query}: [Mock Data]"
