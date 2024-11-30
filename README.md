# **WovenSnake** 🐍  
**WovenSnake** is a Python package manager built with Rust, designed to revolutionize dependency management by introducing a unique, modular, and efficient ecosystem inspired by the elegance of intertwined systems.

---

## **Features**  
- **Simplified Configuration**: Manage dependencies using a JSON/YAML-based configuration file (`wovenpkg.json`) for development, production, and release environments.  
- **Rust-Powered Performance**: Enjoy lightning-fast operations and reliability powered by Rust.  
- **Snake_Island Concept**: Isolate packages and their dependencies, keeping them modular and conflict-free.  
- **Work Directory Management**: Like Node’s `node_modules` or Python’s `venv`, manage your dependencies in isolated work directories for better organization and efficiency.  
- **Future-Proof Design**: Planned features like *Bridge_Snake* to connect shared dependencies efficiently and reduce redundancy.

---

## **Getting Started**  
Follow these steps to set up **WovenSnake** in your project.

### **1. Installation**  
Install **WovenSnake** with a single command using `curl`:  
```bash
curl -fsSL https://raw.githubusercontent.com/jackby03/wovensnake/main/install.sh | bash
```

This script will:  
- Download the latest version of **WovenSnake** from the repository.  
- Build and install it on your system automatically.  

### **2. Initialize Your Project**  
Run the following command in your project directory to create a `wovenpkg.json` configuration file:  
```bash
wovensnake init
```
```sh
/YourProject
├── .venv
├── src
│   └── main.py
├── wovenpkg.json
└── .gitignore
```
### **3. Install Dependencies**  
Add your dependencies to `wovenpkg.json` and install them with:  
```bash
woven install
```
Dependencies will be installed into a dedicated work directory (similar to `venv` in Python or `node_modules` in Node), ensuring isolation and reducing conflicts.

### **4. Update Dependencies**  
To update your dependencies to their latest versions as specified in `wovenpkg.json`, run:  
```bash
woven update
```
This command will fetch and install the latest versions of your dependencies, keeping your project up-to-date.

### **5. Clean Dependencies**  
To remove unused dependencies and clean up your work directory, use:  
```bash
woven clean
```
This command will help maintain a tidy and efficient work environment by removing unnecessary files.

---

## **Configuration**  
**WovenSnake** uses a `wovenpkg.json` file to define dependencies. Here's an example of a basic `wovenpkg.json` file:  
```json
{
  "name": "my-python-project",
  "version": "1.0.0",
  "python_version": "3.10",
  "dependencies": {
    "requests": "2.26.0",
    "numpy": "1.21.0"
  }
}
```
---

## **Planned Features**  
- **Bridge_Snake**: A system to share dependencies between isolated packages, avoiding duplication and enhancing efficiency.  
- **Environment-Specific Settings**: Seamless switching between development, production, and release configurations.  
- **Enhanced Dependency Optimization**: Automatic deduplication and conflict resolution.

---

## **Why WovenSnake?**  
Unlike traditional package managers, **WovenSnake** offers:  
- A fresh perspective on managing Python dependencies with modular and creative concepts.  
- Rust-based performance for faster and more reliable operations.  
- Work directory management, similar to `venv` or `node_modules`, for better dependency isolation.  
- A clear focus on reducing complexity and promoting scalability in large projects.

---

## **Contributing**  
We welcome contributions from the community! If you'd like to contribute:  
1. Fork the repository.  
2. Create a feature branch: `git checkout -b feature/my-feature`.  
3. Commit your changes: `git commit -m "Add my feature"`.  
4. Push to your branch: `git push origin feature/my-feature`.  
5. Open a pull request.

---

## **License**  
This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.
