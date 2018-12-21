#include "jugr.h"

#include "graph.h"
#include <anymap.h>
#include <iniparser.h>

using namespace com;
using namespace ini;
using namespace std;

void test_homelst() {
  std::shared_ptr<com::graph::Node> adotNode =
      com::graph::loadFromADot("graph_jugr_test.dot");
  Anymap adotData;
  // adotData["TSK_FILENAME"] = "rndprm.tsk";
  adotNode->run(adotData);
}

//===============================================================

int main() {
  test_homelst();

  return 0;
}
