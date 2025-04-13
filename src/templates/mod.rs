pub mod api {
    pub fn route(package_name: &str) -> String {
        format!(
            r#"package api

import (
	"{package_name}/core"
	"net/http"
	"time"
)

func Handler(ctx *core.APIContext) {{

	response := map[string]interface{{}}{{
		"message":   "Hello from Go on Airplanes API route!",
		"timestamp": time.Now().Format(time.RFC3339),
		"method":    ctx.Request.Method,
		"path":      ctx.Request.URL.Path,
		"params":    ctx.Params,
		"success":   true,
	}}

	ctx.Success(response, http.StatusOK)
}}
"#
        )
    }
}

pub mod page {
    pub fn normal_page() -> &'static str {
        r#"{{ define "content" }}
<div class="text-center">
    <h2 class="text-2xl font-bold mb-6">Welcome to Go on Airplanes</h2>
    <p class="mb-4">A Go-based fullstack framework with HTML file-based routing</p>
    
    <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mt-10">
        <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
                <div class="text-center">
                    <h3 class="text-lg font-medium">File-based Routing</h3>
                    <p class="mt-2 text-sm text-gray-500">Create pages by adding HTML files in the app directory</p>
                </div>
            </div>
        </div>
        
        <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
                <div class="text-center">
                    <h3 class="text-lg font-medium">Component System</h3>
                    <p class="mt-2 text-sm text-gray-500">Create reusable components with Go templates</p>
                </div>
            </div>
        </div>
        
        <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
                <div class="text-center">
                    <h3 class="text-lg font-medium">Zero Config</h3>
                    <p class="mt-2 text-sm text-gray-500">Just run and go - no build step required</p>
                </div>
            </div>
        </div>
    </div>
    
    <div class="mt-10 mb-10">
        <a href="/dashboard" class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700">
            Go to Dashboard
        </a>
    </div>
    </div>
</div>
{{ end }}"#
    }

    pub fn dynamic_page() -> &'static str {
        r#"{{ define "content" }}
<div>
    <h2 class="text-2xl font-bold mb-6">Dynamic Route Example</h2>
    
    <div class="bg-white overflow-hidden shadow rounded-lg">
        <div class="px-4 py-5 sm:p-6">
            <div>
                <h3 class="text-lg font-medium">Parameter Value</h3>
                <p class="mt-2 text-sm text-gray-500">The value of the <code>id</code> parameter is:</p>
                <div class="mt-4 p-4 bg-gray-100 rounded">
                    <code class="text-lg font-mono">{{.Params.id}}</code>
                </div>
                <p class="mt-4 text-sm text-gray-500">
                    Try changing the URL to see how dynamic routing works!
                </p>
            </div>
        </div>
    </div>
    
    <div class="mt-8">
        <a href="/" class="text-blue-600 hover:text-blue-800">
            &larr; Back to home
        </a>
    </div>
</div>
{{ end }}"#
    }
}

pub mod component {
    pub fn basic_component() -> &'static str {
        r#"{{ define "card" }}
<div class="bg-white overflow-hidden shadow rounded-lg">
    <div class="px-4 py-5 sm:p-6">
        {{.}}
    </div>
</div>
{{ end }}"#
    }
}

pub mod project {
    #[allow(dead_code)]
    pub fn config_json(project_name: &str) -> String {
        format!(
            r#"{{
  "name": "{}",
  "version": "0.1.0",
  "directories": {{
    "appDir": "app",
    "staticDir": "static",
    "layoutPath": "app/layout.html",
    "componentDir": "app/components"
  }}
}}
"#,
            project_name
        )
    }
} 