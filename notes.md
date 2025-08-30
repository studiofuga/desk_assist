Run ollama on docker:

```sh
$ docker run -d --device /dev/kfd --device /dev/dri -v ollama:/root/.ollama -p 11434:11434 --name ollama ollama/ollama:rocm
```

Pull the model:

```shell
$ docker exec -it ollama ollama pull llama3.2
$ docker exec -it ollama ollama pull nomic-embed-text
```

