import { gql, useQuery } from "@apollo/client";
import { Flex, useColorModeValue } from "@chakra-ui/react";
import StatisticBox from "./StatisticBox";

export default function StatisticBar() {
  const now = new Date();
  const begining_time = new Date(now.getFullYear(), now.getMonth(), 1, 0, 0, 1);
  const end_time = new Date(now.getFullYear(), now.getMonth() + 1, 0, 23, 59, 59);

  const { loading, error, data } = useQuery(gql`
  query BAR_STATISTIC($from: Int, $to: Int) {
    statistic(from: $from, to: $to) {
      start
      end
      total: categorySnapshot(categories: ["Assets", "Liabilities"]) {
        summary {
          number
          currency
        }
        detail {
          number
          currency
        }
      }
      monthIncome: distance(accounts: ["Income"]) {
        summary {
          number
          currency
        }
        detail {
          number
          currency
        }
      }
      monthExpense: distance(accounts: ["Expenses"]) {
        summary {
          number
          currency
        }
        detail {
          number
          currency
        }
      }
  
      liability: categorySnapshot(categories: ["Liabilities"]) {
        summary {
          number
          currency
        }
        detail {
          number
          currency
        }
      }
    }
  }
    `, {
    variables: {
      from: Math.round(begining_time.getTime() / 1000),
      to: Math.round(end_time.getTime() / 1000)
    }
  })
  if (loading) return <p>Loading...</p>;
  if (error) return <p>Error :(</p>;
  return (
    <Flex h="20" marginLeft={"var(--chakra-sizes-60)"} p={4} borderBottom="1px" borderBottomColor={useColorModeValue('gray.200', 'gray.700')}>
      <StatisticBox text={"ASSET_BLANACE"} amount={data.statistic.total.summary.number} currency={data.statistic.total.summary.currency} />
      <StatisticBox text={"LIABILITY"} amount={data.statistic.liability.summary.number} currency={data.statistic.liability.summary.currency} negetive />
      <StatisticBox text={"CURRENT_MONTH_INCOME"} amount={data.statistic.monthIncome.summary.number} currency={data.statistic.monthIncome.summary.currency} negetive />
      <StatisticBox text={"CURRENT_MONTH_EXPENSE"} amount={data.statistic.monthExpense.summary.number} currency={data.statistic.monthExpense.summary.currency} />
    </Flex>
  )
}