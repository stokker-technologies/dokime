// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class ClientInspectsABackend
{
  [Fact]
  public async Task BoundCaseReturnsProductValuesAndPreservesRefusal()
  {
    var server = new FakeBackend();
    var headers = new Grpc.Core.Metadata { { "authorization", "Bearer test-token" } };
    var client = new Client(server, headers: headers);
    var testCase = await client.Cases.GetAsync("case-a", "cycle-a");
    Assert.Equal("case-a", testCase.Identifier);
    var failure = await Assert.ThrowsAsync<SdkError>(() => client.Cases.GetAsync("", "cycle-a"));
    Assert.Equal("InvalidArgument", failure.Failure.Code);
    await client.CloseAsync();
    await Assert.ThrowsAsync<InvalidOperationException>(() => client.Cases.GetAsync("case-a"));
  }

  [Theory]
  [InlineData("http://localhost")]
  [InlineData("http://[::1]")]
  [InlineData("http://localhost:0")]
  [InlineData("https://host:443/path")]
  [InlineData("http://user@host:7000")]
  [InlineData(" http://host:7000")]
  public void RefuseNonOriginConnectionTargets(string target) => Assert.Throws<ArgumentException>(() => Client.Endpoint(target));

  [Theory]
  [InlineData("http://localhost:80")]
  [InlineData("http://[::1]:7000")]
  public void AcceptExplicitDefaultAndIpv6Ports(string target) => Assert.Equal(target, Client.Endpoint(target).OriginalString);

  [Theory]
  [InlineData("DOKIME_REMOTE=http://host:7000 # Host", "http://host:7000")]
  [InlineData("DOKIME_REMOTE='path#file.pair'", "path#file.pair")]
  [InlineData("DOKIME_REMOTE=\"a\\tb\"", "a\tb")]
  public void ParseConnectionAssignments(string text, string expected) => Assert.Equal(expected, Configuration.Parse(text, "settings.env")["DOKIME_REMOTE"]);

  [Theory]
  [InlineData("X=one\nX=two")]
  [InlineData("export X=one")]
  [InlineData("X=one\r\r\n")]
  [InlineData("X=\"unclosed")]
  [InlineData("X=\"bad\\q\"")]
  public void RefuseMalformedAssignments(string text) => Assert.Throws<ArgumentException>(() => Configuration.Parse(text, "settings.env"));
}
