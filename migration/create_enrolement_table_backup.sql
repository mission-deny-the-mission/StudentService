CREATE TABLE IF NOT EXISTS Enrolement(
                                         StudentID INT NOT NULL,
                                         CourseID INT NOT NULL,
                                         EnrolementDate TEXT,
                                         PRIMARY KEY (StudentID, CourseID),
    FOREIGN KEY (StudentID) REFERENCES Student(Id),
    FOREIGN KEY (CourseID) REFERENCES Course(Id)
    );

PRAGMA TABLE_INFO(Enrolement);