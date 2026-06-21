**Template de PDF de Contas a Pagar**

**Seção 1: Informações Gerais**
==============================

* **Nome da Empresa**: 🏢 Nome da empresa
* **Endereço**: 📍 Endereço da empresa
* **CPF/CNPJ**: 👥 CPF/CNPJ da empresa

**Seção 2: Fatura**
==================

* **Número da Fatura**: 📝 Número da fatura
* **Data de Emissão**: 📆 Data de emissão da fatura
* **Data de Vencimento**: 📆 Data de vencimento da fatura

**Seção 3: Detalhes da Fatura**
=============================

* **Descrição do Serviço**: 📝 Descrição do serviço prestado
* **Valor do Serviço**: 💸 Valor do serviço prestado
* **Total da Fatura**: 💸 Total da fatura

**Seção 4: Contas a Pagar**
=========================

| **Nome do Fornecedor** | **Valor da Conta** | **Data de Vencimento** |
| --- | --- | --- |
| Fornecedor 1 | 100,00 | 30/06/2023 |
| Fornecedor 2 | 200,00 | 15/07/2023 |
| Fornecedor 3 | 300,00 | 31/07/2023 |

**Seção 5: Total da Fatura**
==========================

* **Total das Contas a Pagar**: 💸 Total das contas a pagar
* **Valor Total da Fatura**: 💸 Valor total da fatura

**Código para Geração do PDF** (Python)
```python
from fpdf import FPDF

class ContaAPagar:
    def __init__(self, nome, valor, data_vencimento):
        self.nome = nome
        self.valor = valor
        self.data_vencimento = data_vencimento

class Fatura:
    def __init__(self, numero, data_emissao, data_vencimento):
        self.numero = numero
        self.data_emissao = data_emissao
        self.data_vencimento = data_vencimento
        self.contas_a_pagar = []

    def add_conta(self, conta):
        self.contas_a_pagar.append(conta)

    def calcular_total(self):
        return sum(conta.valor for conta in self.contas_a_pagar)

pdf = FPDF()

# Seção 1: Informações Gerais
pdf.add_page()
pdf.set_font("Arial", size=12)
pdf.cell(200, 10, txt="Nome da Empresa: Nome da empresa", ln=True, align='L')
pdf.cell(200, 10, txt="Endereço: Endereço da empresa", ln=True, align='L')
pdf.cell(200, 10, txt="CPF/CNPJ: CPF/CNPJ da empresa", ln=True, align='L')

# Seção 2: Fatura
pdf.ln(10)
pdf.cell(200, 10, txt="Número da Fatura: Número da fatura", ln=True, align='L')
pdf.cell(200, 10, txt="Data de Emissão: Data de emissão da fatura", ln=True, align='L')
pdf.cell(200, 10, txt="Data de Vencimento: Data de vencimento da fatura", ln=True, align='L')

# Seção 3: Detalhes da Fatura
pdf.ln(10)
pdf.cell(200, 10, txt="Descrição do Serviço: Descrição do serviço prestado", ln=True, align='L')
pdf.cell(200, 10, txt="Valor do Serviço: Valor do serviço prestado", ln=True, align='L')
pdf.cell(200, 10, txt="Total da Fatura: Total da fatura", ln=True, align='L')

# Seção 4: Contas a Pagar
pdf.ln(10)
pdf.cell(0, 10, txt="Nome do Fornecedor\tValor da Conta\tData de Vencimento", ln=True, align='L')
for conta in fatura.contas_a_pagar:
    pdf.cell(0, 10, txt=f"{conta.nome}\t{conta.valor:.2f}\t{conta.data_vencimento}", ln=True, align='L')

# Seção 5: Total da Fatura
pdf.ln(10)
pdf.cell(200, 10, txt="Total das Contas a Pagar: Total das contas a pagar", ln=True, align='L')
pdf.cell(200, 10, txt=f"Valor Total da Fatura: {fatura.calcular_total():.2f}", ln=True, align='L')

pdf.output("contas_a_pagar.pdf")

# Você gostaria de executar o código para gerar o PDF? (sim/não)