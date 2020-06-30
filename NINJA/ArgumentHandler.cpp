#include "ArgumentHandler.hpp"

ArgumentHandler::ArgumentHandler(char *argv[], int argc) {
    this->abort = false;
    this->alphabetType = null;
    this->correctionType = not_assigned;
    this->inputType = alignment;
    this->method.assign("default");
    this->njTmpDir = "";
    this->numThreads = 0;
    this->outputFile = stdout;
    this->outputType = tree;
    this->useSSE = true;

    if (argc == 1) {
        printf("Use --help (or -h) to see possible arguments.\n");
        Exception::critical();
    }
    for (int i = 1; i < argc; i++) {
        std::string x, y;
        x.assign(argv[i]);
        if (i < argc - 1) y.assign(argv[i + 1]);
        if (!x.compare("--in") || !x.compare("-i")) {
            if (i < argc - 1) {
                this->inputFile.assign(argv[i + 1]);
                i++;
            } else {
                fprintf(stderr, "No file specified.\n");
                Exception::critical();
            }
        } else if (!x.compare("--out") || !x.compare("-o")) {
            if (i < argc - 1) {
                FILE *out = fopen(argv[i + 1], "w+");
                i++;
                if (out == NULL) {
                    fprintf(stderr, "Error opening file");
                    Exception::critical();
                }
                this->outputFile = out;
            } else {
                fprintf(stderr, "No file specified.\n");
                Exception::critical();
            }
        } else if (!x.compare("--method") || !x.compare("-m")) {
            if (i < argc - 1) {
                if (!y.compare("default")) {
                    //nothing happens, default is inmem
                } else if (!y.compare("extmem")) {
                    this->method.assign("extmem");
                } else if (!y.compare("inmem")) {
                    //nothing happens, default is inmem
                } else {
                    fprintf(stderr, "Invalid method: available methods are  'default', 'inmem', and 'extmem'.");
                    Exception::critical();
                }
                i++;
            } else {
                fprintf(stderr, "No method specified.\n");
                Exception::critical();
            }
        } else if (!x.compare("--in_type") || !x.compare("-it")) {
            if (i < argc - 1) {
                if (!y.compare("a")) {
                    this->inputType = alignment;
                } else if (!y.compare("d")) {
                    this->inputType = distance;
                } else {
                    fprintf(stderr, "Invalid in_type:  try  'a' or 'd'.");
                    Exception::critical();
                }
                i++;
            } else {
                fprintf(stderr, "No input type specified.\n");
                Exception::critical();
            }
        } else if (!x.compare("--out_type") || !x.compare("-ot")) {
            if (i < argc - 1) {
                if (!y.compare("d")) {
                    this->outputType = dist;
                } else if (!y.compare("t")) {
                    this->outputType = tree;
                } else {
                    fprintf(stderr, "Invalid out_type:  try  'd' or 't'.");
                    Exception::critical();
                }
                i++;
            } else {
                fprintf(stderr, "No output type specified.\n");
                Exception::critical();
            }
        } else if (!x.compare("--alph_type")) {
            if (i < argc - 1) {
                if (!y.compare("a")) {
                    alphabetType = amino;
                } else if (!y.compare("d")) {
                    alphabetType = dna;
                } else {
                    fprintf(stderr, "Invalid alph_type:  try  'a' or 'd'.");
                    Exception::critical();
                }
                i++;
            } else {
                fprintf(stderr, "No alph_type specified.\n");
                Exception::critical();
            }
        } else if (!x.compare("--corr_type")) {
            if (i < argc - 1) {
                if (!y.compare("n")) {
                    correctionType = none;
                } else if (!y.compare("j")) {
                    correctionType = JukesCantor;
                } else if (!y.compare("k")) {
                    correctionType = Kimura2;
                } else if (!y.compare("s")) {
                    correctionType = FastTree;
                } else {
                    fprintf(stderr, "Invalid ut_type:  try  'n', 'j', 'k', or 's'.");
                    Exception::critical();
                }
                i++;
            } else {
                fprintf(stderr, "No ut_type specified.\n");
                Exception::critical();
            }
        } else if (!x.compare("--quiet")) {
            //TreeBuilder.verbose = 0;
        } else if (!x.compare("--tmp_dir") || !x.compare("-t")) {
            if (i < argc - 1) {
                this->njTmpDir = argv[i + 1];
                if (this->njTmpDir.empty()) {
                    fprintf(stderr, "No temporary directory specified.\n");
                    Exception::critical();
                }
                i++;
            } else {
                fprintf(stderr, "No temporary directory specified.\n");
                Exception::critical();
            }
/*		}else if (!x.compare("--clust_size") || !x.compare("-s")){
			if (i < argc-1){
				int s = *argv[i+1] - '0'; //revisit
				//TreeBuilder.clustCnt = s; TODO:
				i++;
			}else{
				fprintf(stderr,"No clust_size integer specified.\n");
				Exception::critical();
			}
		}else if (!x.compare("--rebuild_step_ratio") || !x.compare("-r")){
			if (i < argc-1){
				int r = strtol(argv[i+1], NULL, 0);
				//TreeBuilder.rebuildStepRatio = r;
				i++;
			}else{
				fprintf(stderr,"No rebuild_step_ratio integer specified.\n");
				Exception::critical();
			}*/
        } else if (!x.compare("--threads") | !x.compare("-T")) {
            int r = strtol(argv[i + 1], NULL, 0);
            this->numThreads = r;
            i++;
        } else if (!x.compare("--NOSSE")) {
            this->useSSE = false;
        } else if (!x.compare("--help") || !x.compare("-h")) {
            printf("Arguments: \n");
            printf("--help (or -h) to display this help\n--in (or -i) filename\n--out (or -o) filename\n--method (or -m)  [inmem | extmem] (default inmem)\n");
            printf("--in_type type [a | d] (default a)\n--out_type type [t | d] (default t)\n--corr_type type [n | j | k | s]\n");
            printf("--threads (or -T) num_threads\nFor more information, check the README file.\n");
            this->abort = true;
        } else if (!x.compare("--version")) {
            printf("Version 0.95\n");
            this->abort = true;
        } else {
            fprintf(stderr, "Invalid argument, ignored.\n");
        }
    }
}

bool ArgumentHandler::getAbort() const {
    return this->abort;
}

std::string ArgumentHandler::getInputFile() const {
    return this->inputFile;
}

ArgumentHandler::InputType ArgumentHandler::getInputType() const {
    return this->inputType;
}

std::string ArgumentHandler::getMethod() const {
    return this->method;
}

int ArgumentHandler::getNumThreads() const {
    return this->numThreads;
}

std::string ArgumentHandler::getNJTmpDir() const {
    return this->njTmpDir;
}

ArgumentHandler::OutputType ArgumentHandler::getOutputType() const {
    return this->outputType;
}

FILE *ArgumentHandler::getOutputFile() const {
    return this->outputFile;
}

ArgumentHandler::AlphabetType ArgumentHandler::getAlphabetType() const {
    return this->alphabetType;
}

ArgumentHandler::CorrectionType ArgumentHandler::getCorrectionType() const {
    return this->correctionType;
}

bool ArgumentHandler::argumentTest() {
    return true;
}

bool ArgumentHandler::getUseSSE() const {
    return this->useSSE;
}
