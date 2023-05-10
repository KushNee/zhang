import { Card, createStyles, Text } from '@mantine/core';
import { useTranslation } from 'react-i18next';
import Amount from './Amount';


const useStyles = createStyles((theme) => ({
  card: {
    backgroundColor: theme.colorScheme === 'dark' ? theme.colors.dark[7] : theme.white,
  },

  label: {
    fontFamily: `Greycliff CF, ${theme.fontFamily}`,
    fontSize: theme.fontSizes.sm,
    color: theme.colors.gray[7],
    lineHeight: 1,
  },

  lead: {
    fontWeight: 700,
    fontSize: theme.fontSizes.xl * 1.15,
    lineHeight: 1,
  },
}));


interface Props {
  text: string;
  amount: string;
  currency: string;
  detail?: any;
  negetive?: boolean;
  hint?: string;
}
export default function StatisticBox({ text, amount, currency, negetive, hint }: Props) {
  const { t } = useTranslation();
  const { classes } = useStyles();
  const displayBox = (
    <Card withBorder px="xl" py="lg" shadow="sm" radius="sm" mt="sm" className={classes.card}>
      <div>
        <Text className={classes.lead}>
          <Amount amount={amount} negetive={negetive} currency={currency} />
        </Text>
        <Text className={classes.label}>
          {t(text)}
        </Text>
      </div>
    </Card>

  );
  return displayBox;
}
