import { Badge, Container, Group, Stack, Table, Tabs, Text, Title, createStyles, px } from '@mantine/core';
import { IconMessageCircle, IconPhoto, IconSettings } from '@tabler/icons';
import { format } from 'date-fns';
import { useParams } from 'react-router';
import useSWR from 'swr';
import { fetcher } from '..';
import AccountBalanceCheckLine from '../components/AccountBalanceCheckLine';
import AccountDocumentUpload from '../components/AccountDocumentUpload';
import Amount from '../components/Amount';
import LoadingComponent from '../components/basic/LoadingComponent';
import PayeeNarration from '../components/basic/PayeeNarration';
import AccountDocumentLine from '../components/documentLines/AccountDocumentLine';
import { AccountInfo, AccountJournalItem, Document } from '../rest-model';

const useStyles = createStyles((theme) => ({
  calculatedAmount: {
    fontSize: px(theme.fontSizes.xl) * 1.1,
    fontWeight: 500,
  },
  detailAmount: {
    fontSize: px(theme.fontSizes.lg),
  },
}));

function SingleAccount() {
  let { accountName } = useParams();
  const { classes } = useStyles();

  const { data: account, error } = useSWR<AccountInfo>(`/api/accounts/${accountName}`, fetcher);

  if (error) return <div>failed to load</div>;
  if (!account) return <div>{error}</div>;
  return (
    <Container fluid>
      <Group position="apart" py="sm" align="baseline">
        <Stack>
          <Title order={2}>
            <Badge>{account.status}</Badge> {account.alias ?? account.name}
          </Title>
          {!!account.alias && <Title order={4}>{account.name}</Title>}
        </Stack>
        <Stack align="end" spacing="xs">
          <Group className={classes.calculatedAmount}>
            {Object.keys(account.amount.detail).length > 1 && <Text>≈</Text>}
            <Amount amount={account.amount.calculated.number} currency={account.amount.calculated.commodity}></Amount>
          </Group>
          {Object.keys(account.amount.detail).length > 1 && (
            <>
              {Object.entries(account.amount.detail).map(([key, value]) => (
                <Amount key={key} className={classes.detailAmount} amount={value} currency={key}></Amount>
              ))}
            </>
          )}
        </Stack>
      </Group>
      <Tabs defaultValue="journals" mt="lg">
        <Tabs.List>
          <Tabs.Tab value="journals" icon={<IconPhoto size={14} />}>
            Journals
          </Tabs.Tab>
          <Tabs.Tab value="documents" icon={<IconMessageCircle size={14} />}>
            Documents
          </Tabs.Tab>
          <Tabs.Tab value="settings" icon={<IconSettings size={14} />}>
            Settings
          </Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="journals" pt="xs">
          <Table verticalSpacing="xs" highlightOnHover>
            <thead>
              <tr>
                <th>Date</th>
                <th>Payee & Narration</th>
                <th style={{ textAlign: 'right' }}>Change Amount</th>
                <th style={{ textAlign: 'right' }}>After Change Amount</th>
              </tr>
            </thead>
            <tbody>
              <LoadingComponent
                url={`/api/accounts/${accountName}/journals`}
                skeleton={<div>loading</div>}
                render={(data: AccountJournalItem[]) => (
                  <>
                    {data.map((item) => (
                      <tr>
                        <td>{format(new Date(item.datetime), 'yyyy-MM-dd hh:mm:ss')}</td>
                        <td>
                          <PayeeNarration payee={item.payee} narration={item.narration} />
                        </td>
                        <td style={{ textAlign: 'right' }}>
                          <Amount amount={item.inferred_unit_number} currency={item.inferred_unit_commodity} />
                        </td>
                        <td style={{ textAlign: 'right' }}>
                          <Amount amount={item.account_after_number} currency={item.account_after_commodity} />
                        </td>
                      </tr>
                    ))}
                  </>
                )}
              />
            </tbody>
          </Table>
        </Tabs.Panel>

        <Tabs.Panel value="documents" pt="xs">
          <LoadingComponent
            url={`/api/accounts/${accountName}/documents`}
            skeleton={<div>loading</div>}
            render={(data: Document[]) => (
              <>
                <AccountDocumentUpload url={`/api/accounts/${accountName}/documents`} />
                {data.map((document, idx) => (
                  <AccountDocumentLine key={idx} {...document} />
                ))}
              </>
            )}
          ></LoadingComponent>
        </Tabs.Panel>

        <Tabs.Panel value="settings" pt="xs">
          <Table verticalSpacing="xs" highlightOnHover>
            <thead>
              <tr>
                <th>Currency</th>
                <th>Current Balance</th>
                <th>Latest Balance Time</th>
                <th>Pad Account</th>
                <th>Distanation</th>
              </tr>
            </thead>
            <tbody>
              {Object.entries(account?.amount.detail ?? []).map(([commodity, amount], idx) => (
                <AccountBalanceCheckLine currentAmount={amount} commodity={commodity} accountName={account.name} />
              ))}
            </tbody>
          </Table>
        </Tabs.Panel>
      </Tabs>
    </Container>
  );
}

export default SingleAccount;
