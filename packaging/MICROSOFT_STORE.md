# Publicar o Nebulance na Microsoft Store

Passo a passo para levar o jogo do repositório até a loja. O que já está pronto no repositório:

| O quê | Onde |
|---|---|
| Manifesto do pacote MSIX | `packaging/msix/AppxManifest.xml` |
| Identidade do Partner Center (falta preencher) | `packaging/msix/identity.json` |
| Ícones e tiles do Menu Iniciar (77 imagens) | `packaging/msix/Assets/` (gerados com `nebulance --export-store-assets`) |
| Script que gera o `.msix` | `packaging/build-msix.ps1` |
| Build na nuvem (GitHub Actions) | `.github/workflows/msix.yml` |
| Textos da página da loja (EN e PT-BR) | `packaging/store/LISTING.md` |
| Screenshots 1920×1080 (EN e PT-BR) | `packaging/store/screenshots/` |
| Capa 1:1, pôster 2:3 e arte de destaque 16:9 | `packaging/store/art/` |
| Política de privacidade | `PRIVACY.md` |

No pacote da loja o jogo salva o progresso na pasta do próprio pacote
(`%LOCALAPPDATA%\Packages\<nome do pacote>\LocalState`), que o Windows apaga ao desinstalar. O
rodapé das Configurações mostra o caminho exato, e é lá que fica a pasta `music` de músicas
personalizadas. A versão do instalador Inno Setup continua usando `%APPDATA%\Nebulance`.

---

## 1. Reservar o nome

1. Entre no [Partner Center](https://partner.microsoft.com/dashboard/apps-and-games/overview).
2. **Apps e jogos > + Novo produto > App MSIX ou PWA**. Não escolha "App EXE ou MSI": esse tipo
   não aceita o pacote `.msix`.
3. Digite **Nebulance** e clique em **Verificar disponibilidade** e depois em **Reservar nome do
   produto**.

## 2. Preencher a identidade do pacote

1. No produto, abra **Gerenciamento de produtos > Identidade do produto**.
2. Copie os três valores para `packaging/msix/identity.json`:

   | Partner Center | `identity.json` |
   |---|---|
   | `Package/Identity/Name` | `IdentityName` |
   | `Package/Identity/Publisher` | `Publisher` (começa com `CN=`) |
   | `Package/Properties/PublisherDisplayName` | `PublisherDisplayName` |

3. Faça commit. Esses valores não são secretos.

Enquanto o arquivo tiver os valores `PREENCHA...`, o pacote funciona para testes locais, mas a loja
recusa o envio.

## 3. Gerar o pacote

**No seu PC (Windows):**

```powershell
winget install Microsoft.WindowsSDK.10.0.26100
powershell -ExecutionPolicy Bypass -File packaging\build-msix.ps1
```

O resultado fica em `dist\Nebulance_<versão>_x64.msix`.

**Ou na nuvem:** no GitHub, abra **Actions > MSIX package > Run workflow**. O workflow também roda
sozinho quando você cria uma tag `v*`, por exemplo `v1.0.0`. Baixe o `.msix` nos *Artifacts* da
execução.

## 4. Testar antes de enviar

1. Gere uma versão assinada para teste:

   ```powershell
   powershell -ExecutionPolicy Bypass -File packaging\build-msix.ps1 -TestSign
   ```

2. Siga as duas linhas que o script imprime: importar o certificado de teste (uma vez, em um
   PowerShell de administrador) e depois `Add-AppxPackage`.
3. Abra o Nebulance pelo Menu Iniciar e confira:
   - o ícone e o tile no Menu Iniciar e na barra de tarefas;
   - se o progresso é salvo depois de fechar e abrir de novo;
   - a tela cheia (F11), o controle e o co-op.
4. Rode o **Windows App Certification Kit**, que faz os mesmos testes técnicos da loja:

   ```powershell
   & "${env:ProgramFiles(x86)}\Windows Kits\10\App Certification Kit\appcert.exe" test -appxpackagepath "dist\Nebulance_0.1.0.0_x64.msix" -reportoutputpath "dist\wack.xml"
   ```

5. Desinstale a versão de teste (`Get-AppxPackage *Nebulance* | Remove-AppxPackage`) e gere o pacote
   final **sem** `-TestSign`. A loja assina o pacote ela mesma.

## 5. Política de privacidade

A loja pede uma URL pública. Opções:

- **Repositório público:** use o link do arquivo no GitHub,
  `https://github.com/<usuário>/<repositório>/blob/main/PRIVACY.md`.
- **Repositório privado:** publique o `PRIVACY.md` em um gist público ou no GitHub Pages e use esse
  link.

## 6. Preencher o envio no Partner Center

Em **Envios > Iniciar envio**:

**Preços e disponibilidade**
- Mercados: todos, ou só os que você quiser.
- Preço: gratuito ou pago. Se for pago, vale ativar uma avaliação gratuita.

**Propriedades**
- Categoria: **Jogos**. Gêneros (até três): **Jogos de tiros**, **Clássicos** e **Ação e aventura**.
- Requisitos de sistema: **Teclado** obrigatório; **Controle Xbox** recomendado.
- Recursos do jogo: **Um jogador** e **Multijogador local / co-op local (2 jogadores)**.
- Política de privacidade: a URL do passo 5.

**Classificação etária (questionário IARC)**. Respostas que descrevem o jogo como ele é hoje:
- Violência: **sim**, violência de fantasia contra criaturas não humanas (insetos espaciais e naves),
  sem sangue nem morte realista.
- Sangue, conteúdo sexual, linguagem imprópria, drogas: **não**.
- Apostas: o jogo **não** tem apostas com dinheiro real. A "Caixa Misteriosa" da loja é comprada só
  com moedas ganhas jogando. Se o questionário perguntar sobre itens aleatórios comprados com moeda
  do jogo, responda **sim**, e **não** para dinheiro real.
- Interação entre usuários, compartilhamento de localização, compras digitais: **não**.

**Pacotes**
- Envie o `.msix` gerado no passo 3. Família de dispositivos: **Desktop**.

**Listagens da loja** (uma para **Inglês (Estados Unidos)** e outra para **Português (Brasil)**)
- Textos: copie de `packaging/store/LISTING.md`.
- Screenshots: `packaging/store/screenshots/en-US` e `.../pt-BR`.
- Imagens da loja: capa 1:1, pôster 2:3 e arte de destaque 16:9 de `packaging/store/art/`.

Depois é só clicar em **Enviar para a Store**. A certificação costuma levar de algumas horas a
três dias úteis.

## 7. Atualizações

1. Aumente a versão nos três lugares:
   - `Cargo.toml` (`version`);
   - `assets/nebulance.rc` (`FILEVERSION`, `PRODUCTVERSION` e os textos de versão);
   - `installer/nebulance.iss` (`AppVersion`).

   O script do MSIX lê a versão do `Cargo.toml`. A loja exige que cada envio tenha versão maior que
   o anterior.
2. Gere o pacote de novo (passo 3) e crie um novo envio com o novo `.msix` e o texto de
   "Novidades desta versão".
