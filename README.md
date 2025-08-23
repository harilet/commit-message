# Commit Message
## Introduction

Commit Message is a CLI tool that allows you to easily create commit messages in your Git repository. It provides a simple way to create good commit messages, while also helping you to stick to the guidelines of your project.

## Installation

### **1. Clone the Repository**
```bash
git clone https://github.com/harilet/commit-message.git
cd commit-message
```

### **2. Build the Project**
Use the following commands to compile and check for errors:
```bash
cargo build --release
```

### **3. Make the Project Accessable**
Move the binary file to your PATH so can be accessable from anywhere in your system

### **4. Ollama Server**
- **What it is**: Ollama is a tool designed to run large language models (LLMs) locally on a user's machine

Install ollama by going to [https://ollama.com/download](https://ollama.com/download) and installing the appropriate version for your OS.

Note: Note down the url that ollama is running on.

---

### **5. Model**
- **What it is**: The "brain" of the system. It's a large AI that understands programming and writing.

Install the model by going to [https://ollama.com/models](https://ollama.com/models) and downloading the appropriate version for your system.

a general guideline for how to select model is finding a model that is similar in size to your available RAM. smaller the model size faster the model will be.

note: Note down the llm name that you want to use.

---

### **6. System Prompts**
- **What they are**: Instructions the model follows and acts according to.

There is some system prompts already available but you can also add your own.

---

### **7. Setting up config file**
create a config file using the following instructions:
- name the file `config.json`
- path should be the same directory as `commit-message` binary.
- should follow the format below:
```json
{
  "ollama_server": <the url of ollama server>,
  "model": <model name you want to use>,
  "system_prompts":[
    <multiple system prompts>
  ],
  "commit_message":[
    <instructions for creating commit message>
  ]
}
```

## Usage
Once installed, you can use Commit Message by running the following command in your Git repository:

`
commit-message -h
`

This will display a help menu with information about the available options and commands.