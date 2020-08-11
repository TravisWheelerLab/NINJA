#include "ExceptionHandler.hpp"
#include "DistanceCalculator.hpp"

class DistanceReaderExtMem {

public:
    DistanceReaderExtMem(std::string fileName);

    //DistanceReaderExtMem (float** inD );
    DistanceReaderExtMem(DistanceCalculator *distCalc, int K);


    static const int numPages = 10;
    int K = 0;
    //BufferedReader r; not sure if it will be used, revisit
    const int floatSize = 4;
    float **inD = nullptr;
    DistanceCalculator *distCalc = nullptr;

    int
    read(std::string **names, float *R, FILE *diskD, float **memD, int memDRowsize, int rowLength, int pageBlockSize);

private:
    float atoi(char *in, int end);

};
