/**CFile****************************************************************

  FileName    [acecXor.c]

  SystemName  [ABC: Logic synthesis and verification system.]

  PackageName [CEC for arithmetic circuits.]

  Synopsis    [Detection of XOR trees.]

  Author      [Alan Mishchenko]
  
  Affiliation [UC Berkeley]

  Date        [Ver. 1.0. Started - June 20, 2005.]

  Revision    [$Id: acecXor.c,v 1.00 2005/06/20 00:00:00 alanmi Exp $]

***********************************************************************/

#include "acecInt.h"
#include "misc/vec/vecWec.h"
#include "misc/extra/extra.h"

ABC_NAMESPACE_IMPL_START


// label file type
#define JSON (0)
#define CSV (1)
////////////////////////////////////////////////////////////////////////
///                        DECLARATIONS                              ///
////////////////////////////////////////////////////////////////////////


////////////////////////////////////////////////////////////////////////
///                     FUNCTION DEFINITIONS                         ///
////////////////////////////////////////////////////////////////////////

/**Function*************************************************************

  Synopsis    []

  Description []
               
  SideEffects []

  SeeAlso     []

***********************************************************************/
void Acec_CheckXors( Gia_Man_t * p, Vec_Int_t * vXors )
{
    Gia_Obj_t * pFan0, * pFan1;
    Vec_Int_t * vCount2 = Vec_IntAlloc( Gia_ManObjNum(p) );
    int i, Entry, Count = 0;
    for ( i = 0; 4*i < Vec_IntSize(vXors); i++ )
        if ( Vec_IntEntry(vXors, 4*i+3) == 0 )
            Vec_IntAddToEntry( vCount2, Vec_IntEntry(vXors, 4*i), 1 );
    Vec_IntForEachEntry( vCount2, Entry, i )
        if ( Entry > 1 )
            printf( "*** Obj %d has %d two-input XOR cuts.\n", i, Entry ), Count++;
        else if ( Entry == 1 && Gia_ObjRecognizeExor(Gia_ManObj(p, i), &pFan0, &pFan1) )
            printf( "*** Obj %d cannot be recognized as XOR.\n", i );
    if ( Count == 0 )
        printf( "*** There no multiple two-input XOR cuts.\n" );
    Vec_IntFree( vCount2 );
}

/**Function*************************************************************

  Synopsis    []

  Description []
               
  SideEffects []

  SeeAlso     []

***********************************************************************/
Vec_Int_t * Acec_OrderTreeRoots( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Int_t * vXorRoots, Vec_Int_t * vRanks )
{
    Vec_Int_t * vOrder = Vec_IntAlloc( Vec_IntSize(vXorRoots) );
    Vec_Int_t * vMove = Vec_IntStartFull( Vec_IntSize(vXorRoots) );
    int i, k, Entry, This;
    // iterate through adders and for each try mark the next one
    for ( i = 0; 6*i < Vec_IntSize(vAdds); i++ )
    {
        int Node = Vec_IntEntry(vAdds, 6*i+4);
        if ( Vec_IntEntry(vRanks, Node) == -1 )
            continue;
        for ( k = 0; k < 3; k++ )
        {
            int Fanin = Vec_IntEntry(vAdds, 6*i+k);
            if ( Vec_IntEntry(vRanks, Fanin) == -1 )
                continue;
            //printf( "%4d:  %2d -> %2d\n", Node, Vec_IntEntry(vRanks, Node), Vec_IntEntry(vRanks, Fanin) );
            Vec_IntWriteEntry( vMove, Vec_IntEntry(vRanks, Node), Vec_IntEntry(vRanks, Fanin) );
        }
    }
//Vec_IntPrint( vMove );
    // find reodering
    Vec_IntForEachEntry( vMove, Entry, i )
        if ( Entry == -1 && Vec_IntFind(vMove, i) >= 0 )
            break;
    assert( i < Vec_IntSize(vMove) );
    while ( 1 )
    {
        Vec_IntPush( vOrder, Vec_IntEntry(vXorRoots, i) );
        Entry = i;
        Vec_IntForEachEntry( vMove, This, i )
            if ( This == Entry )
                break;
        if ( i == Vec_IntSize(vMove) )
            break;
    }
    Vec_IntFree( vMove );
//Vec_IntPrint( vOrder );
    return vOrder;
}
                       
/**Function*************************************************************

  Synopsis    []

  Description []
               
  SideEffects []

  SeeAlso     []

***********************************************************************/
// marks XOR outputs
Vec_Bit_t * Acec_MapXorOuts( Gia_Man_t * p, Vec_Int_t * vXors )
{
    Vec_Bit_t * vMap = Vec_BitStart( Gia_ManObjNum(p) ); int i;
    for ( i = 0; 4*i < Vec_IntSize(vXors); i++ )
        Vec_BitWriteEntry( vMap, Vec_IntEntry(vXors, 4*i), 1 );
    return vMap;
}
// marks XOR outputs participating in trees
Vec_Bit_t * Acec_MapXorOuts2( Gia_Man_t * p, Vec_Int_t * vXors, Vec_Int_t * vRanks )
{
    Vec_Bit_t * vMap = Vec_BitStart( Gia_ManObjNum(p) ); int i;
    for ( i = 0; 4*i < Vec_IntSize(vXors); i++ )
        if ( Vec_IntEntry(vRanks, Vec_IntEntry(vXors, 4*i)) != -1 )
            Vec_BitWriteEntry( vMap, Vec_IntEntry(vXors, 4*i), 1 );
    return vMap;
}
// marks MAJ outputs
Vec_Bit_t * Acec_MapMajOuts( Gia_Man_t * p, Vec_Int_t * vAdds )
{
    Vec_Bit_t * vMap = Vec_BitStart( Gia_ManObjNum(p) ); int i;
    for ( i = 0; 6*i < Vec_IntSize(vAdds); i++ )
        Vec_BitWriteEntry( vMap, Vec_IntEntry(vAdds, 6*i+4), 1 );
    return vMap;
}
// marks MAJ outputs participating in trees
Vec_Int_t * Acec_MapMajOuts2( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Int_t * vRanks )
{
    Vec_Int_t * vMap = Vec_IntStartFull( Gia_ManObjNum(p) ); int i;
    for ( i = 0; 6*i < Vec_IntSize(vAdds); i++ )
        if ( Vec_IntEntry(vRanks, Vec_IntEntry(vAdds, 6*i+4)) != -1 )
            Vec_IntWriteEntry( vMap, Vec_IntEntry(vAdds, 6*i+4), i );
    return vMap;
}
// marks nodes appearing as fanins to XORs
Vec_Bit_t * Acec_MapXorIns( Gia_Man_t * p, Vec_Int_t * vXors )
{
    Vec_Bit_t * vMap = Vec_BitStart( Gia_ManObjNum(p) ); int i;
    for ( i = 0; 4*i < Vec_IntSize(vXors); i++ )
    {
        Vec_BitWriteEntry( vMap, Vec_IntEntry(vXors, 4*i+1), 1 );
        Vec_BitWriteEntry( vMap, Vec_IntEntry(vXors, 4*i+2), 1 );
        Vec_BitWriteEntry( vMap, Vec_IntEntry(vXors, 4*i+3), 1 );
    }
    return vMap;
}
// collects XOR roots (XOR nodes not appearing as fanins of other XORs)
Vec_Int_t * Acec_FindXorRoots( Gia_Man_t * p, Vec_Int_t * vXors )
{
    Vec_Bit_t * vMapXorIns = Acec_MapXorIns( p, vXors );
    Vec_Int_t * vXorRoots = Vec_IntAlloc( 100 );  int i;
    for ( i = 0; 4*i < Vec_IntSize(vXors); i++ )
        if ( !Vec_BitEntry(vMapXorIns, Vec_IntEntry(vXors, 4*i)) )
            Vec_IntPushUniqueOrder( vXorRoots, Vec_IntEntry(vXors, 4*i) );
    Vec_BitFree( vMapXorIns );
    return vXorRoots;
}
// collects XOR trees belonging to each of XOR roots
Vec_Int_t * Acec_RankTrees( Gia_Man_t * p, Vec_Int_t * vXors, Vec_Int_t * vXorRoots )
{
    Vec_Int_t * vDoubles = Vec_IntAlloc( 100 );
    int i, k, Entry;
    // map roots into their ranks
    Vec_Int_t * vRanks = Vec_IntStartFull( Gia_ManObjNum(p) ); 
    Vec_IntForEachEntry( vXorRoots, Entry, i )
        Vec_IntWriteEntry( vRanks, Entry, i );
    // map nodes into their ranks
    for ( i = Vec_IntSize(vXors)/4 - 1; i >= 0; i-- )
    {
        int Root = Vec_IntEntry( vXors, 4*i );
        int Rank = Vec_IntEntry( vRanks, Root );
        // skip XORs that are not part of any tree
        if ( Rank == -1 )
            continue;
        // iterate through XOR inputs
        for ( k = 1; k < 4; k++ )
        {
            int Node = Vec_IntEntry( vXors, 4*i+k );
            if ( Node == 0 ) // HA
                continue;
            Entry = Vec_IntEntry( vRanks, Node );
            if ( Entry == Rank ) // the same tree
                continue;
            if ( Entry == -1 )
                Vec_IntWriteEntry( vRanks, Node, Rank );
            else
                Vec_IntPush( vDoubles, Node );

            //if ( Entry != -1 && Gia_ObjIsAnd(Gia_ManObj(p, Node)))
            //printf( "Xor node %d belongs to Tree %d and Tree %d.\n", Node, Entry, Rank );
        }
    }
    // remove duplicated entries
    Vec_IntForEachEntry( vDoubles, Entry, i )
        Vec_IntWriteEntry( vRanks, Entry, -1 );
    Vec_IntFree( vDoubles );
    return vRanks;
}
// collects leaves of each XOR tree
Vec_Wec_t * Acec_FindXorLeaves( Gia_Man_t * p, Vec_Int_t * vXors, Vec_Int_t * vAdds, Vec_Int_t * vXorRoots, Vec_Int_t * vRanks, Vec_Wec_t ** pvAddBoxes )
{
    Vec_Bit_t * vMapXors = Acec_MapXorOuts2( p, vXors, vRanks );
    Vec_Int_t * vMapMajs = Acec_MapMajOuts2( p, vAdds, vRanks );
    int Entry10 = 0; int i10=0;
    Vec_IntForEachEntry(vMapMajs, Entry10, i10)
        printf("vMapMajs %d\n", Entry10);
    Vec_Wec_t * vXorLeaves = Vec_WecStart( Vec_IntSize(vXorRoots) );
    Vec_Wec_t * vAddBoxes  = Vec_WecStart( Vec_IntSize(vXorRoots) );
    int i, k;
    for ( i = 0; 4*i < Vec_IntSize(vXors); i++ )
    {
        int Xor = Vec_IntEntry(vXors, 4*i);
        int Rank = Vec_IntEntry(vRanks, Xor);
        if ( Rank == -1 )
            continue;
        for ( k = 1; k < 4; k++ )
        {
            int Fanin = Vec_IntEntry(vXors, 4*i+k);
            //int RankFanin = Vec_IntEntry(vRanks, Fanin);
            if ( Fanin == 0 )
                continue;
            if ( Vec_BitEntry(vMapXors, Fanin) )
            {
                assert( Rank == Vec_IntEntry(vRanks, Fanin) );
                continue;
            }
//            if ( Vec_BitEntry(vMapXors, Fanin) && Rank == RankFanin )
//                continue;
            if ( Vec_IntEntry(vMapMajs, Fanin) == -1 ) // no adder driving this input
                Vec_WecPush( vXorLeaves, Rank, Fanin );
            else if ( Vec_IntEntry(vRanks, Xor) > 0 ) // save adder box
                Vec_WecPush( vAddBoxes, Rank-1, Vec_IntEntry(vMapMajs, Fanin) );
        }
    }
    Vec_BitFree( vMapXors );
    Vec_IntFree( vMapMajs );
    if ( pvAddBoxes )
        *pvAddBoxes = vAddBoxes;
    return vXorLeaves;
}
void Acec_CheckBoothPPs( Gia_Man_t * p, Vec_Wec_t * vLitLeaves )
{
    Vec_Bit_t * vMarked = Acec_MultMarkPPs( p );
    Vec_Int_t * vLevel;
    int i, k, iLit;
    Vec_WecForEachLevel( vLitLeaves, vLevel, i )
    {
        int CountPI = 0, CountB = 0, CountNB = 0;
        Vec_IntForEachEntry( vLevel, iLit, k )
            if ( !Gia_ObjIsAnd(Gia_ManObj(p, Abc_Lit2Var(iLit))) )
                CountPI++;
            else if ( Vec_BitEntry( vMarked, Abc_Lit2Var(iLit) ) )
                CountB++;
            else
                CountNB++;

        printf( "Rank %2d : Lits = %5d    PI = %d  Booth = %5d  Non-Booth = %5d\n", i, Vec_IntSize(vLevel), CountPI, CountB, CountNB );
    }
    Vec_BitFree( vMarked );
}
Acec_Box_t * Acec_FindBox( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Wec_t * vAddBoxes, Vec_Wec_t * vXorLeaves, Vec_Int_t * vXorRoots )
{
    extern Vec_Int_t * Acec_TreeCarryMap( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Wec_t * vBoxes );
    extern void Acec_TreePhases_rec( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Int_t * vMap, int Node, int fPhase, Vec_Bit_t * vVisit );
    extern void Acec_TreeVerifyPhases( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Wec_t * vBoxes );
    extern void Acec_TreeVerifyPhases2( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Wec_t * vBoxes );

    int MaxRank = Vec_WecSize( vAddBoxes );
    Vec_Bit_t * vVisit  = Vec_BitStart( Vec_IntSize(vAdds)/6 );
    Vec_Bit_t * vIsLeaf = Vec_BitStart( Gia_ManObjNum(p) );
    Vec_Bit_t * vIsRoot = Vec_BitStart( Gia_ManObjNum(p) );
    Vec_Int_t * vLevel, * vLevel2, * vMap;
    int i, j, k, Box, Node;

    Acec_Box_t * pBox = ABC_CALLOC( Acec_Box_t, 1 );
    pBox->pGia        = p;
    pBox->vAdds       = vAddBoxes; // Vec_WecDup( vAddBoxes );
    pBox->vLeafLits   = Vec_WecStart( MaxRank + 0 );
    pBox->vRootLits   = Vec_WecStart( MaxRank + 0 );

    assert( Vec_WecSize(vAddBoxes) == Vec_WecSize(vXorLeaves) );
    assert( Vec_WecSize(vAddBoxes) == Vec_IntSize(vXorRoots) );

    // collect boxes; mark inputs/outputs
    Vec_WecForEachLevel( pBox->vAdds, vLevel, i )
    Vec_IntForEachEntry( vLevel, Box, k )
    {
        Vec_BitWriteEntry( vIsLeaf, Vec_IntEntry(vAdds, 6*Box+0), 1 );
        Vec_BitWriteEntry( vIsLeaf, Vec_IntEntry(vAdds, 6*Box+1), 1 );
        Vec_BitWriteEntry( vIsLeaf, Vec_IntEntry(vAdds, 6*Box+2), 1 );
        Vec_BitWriteEntry( vIsRoot, Vec_IntEntry(vAdds, 6*Box+3), 1 );
        Vec_BitWriteEntry( vIsRoot, Vec_IntEntry(vAdds, 6*Box+4), 1 );
    }
    // sort each level
    Vec_WecForEachLevel( pBox->vAdds, vLevel, i )
        Vec_IntSort( vLevel, 0 );

    // set phases starting from roots
    vMap = Acec_TreeCarryMap( p, vAdds, pBox->vAdds );
    Vec_WecForEachLevelReverse( pBox->vAdds, vLevel, i )
        Vec_IntForEachEntry( vLevel, Box, k )
            if ( !Vec_BitEntry( vIsLeaf, Vec_IntEntry(vAdds, 6*Box+4) ) )
            {
                //printf( "Pushing phase of output %d of box %d\n", Vec_IntEntry(vAdds, 6*Box+4), Box );
                Acec_TreePhases_rec( p, vAdds, vMap, Vec_IntEntry(vAdds, 6*Box+4), Vec_IntEntry(vAdds, 6*Box+2) != 0, vVisit );
            }
    Acec_TreeVerifyPhases( p, vAdds, pBox->vAdds );
    Acec_TreeVerifyPhases2( p, vAdds, pBox->vAdds );
    Vec_BitFree( vVisit );
    Vec_IntFree( vMap );

    // collect inputs/outputs
    Vec_BitWriteEntry( vIsRoot, 0, 1 );
    Vec_WecForEachLevel( pBox->vAdds, vLevel, i )
        Vec_IntForEachEntry( vLevel, Box, j )
        {
            for ( k = 0; k < 3; k++ )
                if ( !Vec_BitEntry( vIsRoot, Vec_IntEntry(vAdds, 6*Box+k) ) )
                    Vec_WecPush( pBox->vLeafLits, i, Abc_Var2Lit(Vec_IntEntry(vAdds, 6*Box+k), Acec_SignBit2(vAdds, Box, k)) );
            for ( k = 3; k < 5; k++ )
                if ( !Vec_BitEntry( vIsLeaf, Vec_IntEntry(vAdds, 6*Box+k) ) )
                    Vec_WecPush( pBox->vRootLits, k == 4 ? i + 1 : i, Abc_Var2Lit(Vec_IntEntry(vAdds, 6*Box+k), Acec_SignBit2(vAdds, Box, k)) );
            if ( Vec_IntEntry(vAdds, 6*Box+2) == 0 && Acec_SignBit2(vAdds, Box, 2) )
                Vec_WecPush( pBox->vLeafLits, i, 1 );
        }
    Vec_BitFree( vIsLeaf );
    Vec_BitFree( vIsRoot );

    // collect last bit
    vLevel  = Vec_WecEntry( pBox->vLeafLits, Vec_WecSize(pBox->vLeafLits)-1 );
    vLevel2 = Vec_WecEntry( vXorLeaves, Vec_WecSize(vXorLeaves)-1 );
    if ( Vec_IntSize(vLevel) == 0 && Vec_IntSize(vLevel2) > 0 )
    {
        Vec_IntForEachEntry( vLevel2, Node, k )
            Vec_IntPush( vLevel, Abc_Var2Lit(Node, 0) );
    }
    vLevel  = Vec_WecEntry( pBox->vRootLits, Vec_WecSize(pBox->vRootLits)-1 );
    Vec_IntFill( vLevel, 1, Abc_Var2Lit(Vec_IntEntryLast(vXorRoots), 0) );

    // sort each level
    Vec_WecForEachLevel( pBox->vLeafLits, vLevel, i )
        Vec_IntSort( vLevel, 0 );
    Vec_WecForEachLevel( pBox->vRootLits, vLevel, i )
        Vec_IntSort( vLevel, 1 );

    //Acec_CheckBoothPPs( p, pBox->vLeafLits );
    return pBox;
}


void Gia_edgelist(Gia_Man_t * p, char *f0, char *f1, char *f2)
{
    int i=0;
    FILE * f_el,  * f_feats;
    f_el = fopen (f0, "w"); 
    f_feats = fopen (f2, "w"); 
    Gia_Obj_t * pObj;
    Gia_Obj_t * pFan0, * pFan1;
    Gia_ManForEachCi( p, pObj, i )
        //fprintf(f_feats, "0,0,0,0,0,0,-1,-1\n", Gia_ObjId(p,pObj));
        fprintf(f_feats, "0,0,0,0\n", Gia_ObjId(p,pObj));
    Gia_ManForEachAnd( p, pObj, i ){
        //printf("%d %d  %d\n", Gia_ObjFaninId0(pObj, Gia_ObjId(p,pObj)), Gia_ObjId(p,pObj), Gia_ObjFaninC0(pObj));
        fprintf(f_el, "%d %d\n", Gia_ObjFaninId0(pObj, Gia_ObjId(p,pObj))-1, Gia_ObjId(p,pObj)-1);
        //printf("%d %d  %d\n", Gia_ObjFaninId1(pObj, Gia_ObjId(p,pObj)), Gia_ObjId(p,pObj), Gia_ObjFaninC1(pObj));
        fprintf(f_el, "%d %d\n", Gia_ObjFaninId1(pObj, Gia_ObjId(p,pObj))-1, Gia_ObjId(p,pObj)-1);
        // feats with complement values
        fprintf(f_feats, "1,1,%d,%d\n", Gia_ObjFaninC0(pObj), Gia_ObjFaninC1(pObj));
        /* feats with truth table
        if (Gia_ObjFaninC0(pObj)==0 && Gia_ObjFaninC1(pObj)==0)
            fprintf(f_feats, "1,1,0,0,0,1,%d,%d\n",Gia_ObjId(p,Gia_ObjFanin0(pObj)),Gia_ObjId(p,Gia_ObjFanin1(pObj)));
        else if (Gia_ObjFaninC0(pObj)==1 && Gia_ObjFaninC1(pObj)==0)
            fprintf(f_feats, "1,1,0,1,0,0,%d,%d\n",Gia_ObjId(p,Gia_ObjFanin0(pObj)),Gia_ObjId(p,Gia_ObjFanin1(pObj)));
        else if (Gia_ObjFaninC0(pObj)==0 && Gia_ObjFaninC1(pObj)==1)
            fprintf(f_feats, "1,1,0,0,1,0,%d,%d\n",Gia_ObjId(p,Gia_ObjFanin0(pObj)),Gia_ObjId(p,Gia_ObjFanin1(pObj)));
        else
            fprintf(f_feats, "1,1,1,0,0,0,%d,%d\n",Gia_ObjId(p,Gia_ObjFanin0(pObj)),Gia_ObjId(p,Gia_ObjFanin1(pObj)));
        */

    }
    Gia_ManForEachCo( p, pObj, i ){
        fprintf(f_el, "%d %d\n", Gia_ObjFaninId0(pObj, Gia_ObjId(p,pObj))-1, Gia_ObjId(p,pObj)-1); 
        fprintf(f_el, "%d %d\n", Gia_ObjFaninId1(pObj, Gia_ObjId(p,pObj))-1, Gia_ObjId(p,pObj)-1); 
        fprintf(f_feats, "0,0,1,1\n", Gia_ObjId(p,pObj));
        //fprintf(f_feats, "0,0,1,1,1,1,-1,-1\n", Gia_ObjId(p,pObj));
    }
    fclose(f_el);
    fclose(f_feats);

}

int isAdds(int id, Vec_Int_t * vAdds)
{
    int i;
    for ( i = 0; 6*i < Vec_IntSize(vAdds); i++ )
    {
        if(id == Vec_IntEntry(vAdds, 6*i+3))
            return 1;
        else if(id == Vec_IntEntry(vAdds, 6*i+4)) // maj output
            return 2;
    }
    return 0;
}
int isVecEntry(int id, Vec_Int_t * vInt)
{
    int i, Entry;
    Vec_IntForEachEntry( vInt, Entry, i ){
         if (Entry == id)
             return 1;
    }
    return 0;
}

Vec_Int_t * extractXor(Vec_Int_t * vAdds)
{
    int i;
    Vec_Int_t * v = Vec_IntAlloc(10);
    for ( i = 0; 6*i < Vec_IntSize(vAdds); i++ )
    {
        Vec_IntPush( v, Vec_IntEntry(vAdds, 6*i+3) );
    }
    return v;
}


Vec_Int_t * extractMaj(Vec_Int_t * vAdds)
{
    int i;
    Vec_Int_t * v = Vec_IntAlloc(10);
    for ( i = 0; 6*i < Vec_IntSize(vAdds); i++ )
    {
        Vec_IntPush( v, Vec_IntEntry(vAdds, 6*i+4) );
    }
    return v;
}




//cunxi


// Multiple label(class) on same nodes if possible 
void Gia_EdgelistMultiLabel( Gia_Man_t * p , char *f0, char *f1, char *f2, int multihot)
{
    extern void Acec_TreeVerifyConnections( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Wec_t * vBoxes );

    abctime clk = Abc_Clock();
    Acec_Box_t * pBox = NULL;
    Vec_Int_t * vXors, * vAdds = Ree_ManComputeCuts( p, &vXors, 0 );
    Vec_Int_t * vTemp, * vXorRoots = Acec_FindXorRoots( p, vXors ); 
    Vec_Int_t * vRanks = Acec_RankTrees( p, vXors, vXorRoots ); 
    Vec_Wec_t * vXorLeaves, * vAddBoxes = NULL; 

    Vec_Int_t * vAdds_copy = Vec_IntDup(vAdds);
    Vec_Int_t * vXors_copy = Vec_IntDup(vXors);
    Vec_IntSort(vAdds, 0);  // sorting in ascending order for common search
    Vec_IntSort(vXors, 0); // sorting in ascending order for common search

    Vec_Int_t * vMaj = extractMaj(vAdds_copy);
    Vec_Int_t * vXors_e = extractXor(vAdds_copy);

    //Vec_IntPrint(vMaj);
    //Vec_IntPrint(vXorRoots);

    Vec_Int_t * vXors_vAdds_share = Vec_IntAlloc( Gia_ManObjNum(p) ); // initialize with upper bound node count
    int common = Vec_IntTwoFindCommon( vXors, vAdds, vXors_vAdds_share);
    if (common > 0){
        //printf("common > 0");
        common = Vec_IntUniqify(vXors_vAdds_share);
    }

    //Vec_IntUniqify(vXors_copy);
    //Vec_IntUniqify(vAdds_copy);
    /*
    printf( "XOR after Cut compute: \n" );
    Vec_IntPrint( vXors_copy );
    printf( "vAdds after Cut compute: \n" );
    Vec_IntPrint( vAdds_copy );
    printf( "XOR roots after reordering: \n" );
    Vec_IntPrint( vXorRoots );
    printf( "xor add share: %d \n" , common );
    if (common > 0){
        Vec_IntPrint( vXors_vAdds_share );
    }
*/
    Gia_ManLevelNum(p);
    Gia_edgelist(p,f0,f1,f2);
    FILE * f_class;
    f_class = fopen (f1, "w"); 
    int i;
    Gia_Obj_t * pObj;
    if(multihot!=1){
	if JSON {
        fprintf(f_class, "{");
	    Gia_ManForEachCi( p, pObj, i )
		    fprintf(f_class, "\"%d\": [0,0,0,0,0,1,0,0], ", Gia_ObjId(p,pObj)-1);
	    Gia_ManForEachAnd(p, pObj, i){
            // MTL label (shared)
		    if ( Vec_IntFind(vXors_vAdds_share, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,1,0,0,0,", Gia_ObjId(p,pObj)-1); //xor and maj 
		    else if ( Vec_IntFind(vAdds_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,0,1,0,0,", Gia_ObjId(p,pObj)-1); // maj
		    else if ( Vec_IntFind(vXors_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,0,0,1,0,", Gia_ObjId(p,pObj)-1); // xor
		    else
			    fprintf(f_class, "\"%d\": [0,1,0,0,0,0,", Gia_ObjId(p,pObj)-1); // and
            // root label
		    if (isAdds(Gia_ObjId(p,pObj), vAdds) == 0)
                fprintf(f_class, "0,1], ", Gia_ObjId(p,pObj)-1); //xor 
            else if(isAdds(Gia_ObjId(p,pObj), vAdds) == 1)
                fprintf(f_class, "1,0], ", Gia_ObjId(p,pObj)-1); // maj
            else
                fprintf(f_class, "0,0], ", Gia_ObjId(p,pObj)-1); // and
   	    } 
	    Gia_ManForEachCo( p, pObj, i )
		    if(  i < Gia_ManCoNum(p) - 1)
			    fprintf(f_class, "\"%d\": [1,0,0,0,0,0,0,0], ", Gia_ObjId(p,pObj)-1);
		    else
			    fprintf(f_class, "\"%d\": [1,0,0,0,0,0,0,0]", Gia_ObjId(p,pObj)-1);

	    fprintf(f_class, "}");
	    fclose(f_class);
		 }

	else {
	    Gia_ManForEachCi( p, pObj, i )
		    fprintf(f_class, "0,0,0,0,0,1,0,0\n", Gia_ObjId(p,pObj)-1);
	    Gia_ManForEachAnd(p, pObj, i){
            // MTL label (shared)
		    if ( Vec_IntFind(vXors_vAdds_share, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "0,0,1,0,0,0,", Gia_ObjId(p,pObj)-1); //xor and maj 
		    else if ( Vec_IntFind(vAdds_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "0,0,0,1,0,0,", Gia_ObjId(p,pObj)-1); // maj
		    else if ( Vec_IntFind(vXors_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "0,0,0,0,1,0,", Gia_ObjId(p,pObj)-1); // xor
		    else
			    fprintf(f_class, "0,1,0,0,0,0,", Gia_ObjId(p,pObj)-1); // and
            // root label
		    if (isAdds(Gia_ObjId(p,pObj), vAdds) == 0)
                fprintf(f_class, "0,1\n", Gia_ObjId(p,pObj)-1); //xor 
            else if(isAdds(Gia_ObjId(p,pObj), vAdds) == 1)
                fprintf(f_class, "1,0\n", Gia_ObjId(p,pObj)-1); // maj
            else
                fprintf(f_class, "0,0\n", Gia_ObjId(p,pObj)-1); // and
   	    } 
	    Gia_ManForEachCo( p, pObj, i )
		    if(  i < Gia_ManCoNum(p) - 1)
			    fprintf(f_class, "1,0,0,0,0,0,0,0\n", Gia_ObjId(p,pObj)-1);
		    else
			    fprintf(f_class, "1,0,0,0,0,0,0,0\n", Gia_ObjId(p,pObj)-1);

	    fclose(f_class);
		 }


    }
    else {
         if JSON {
	    Gia_ManForEachCi( p, pObj, i )
		    fprintf(f_class, "\"%d\": [0,0,0,0,1], ", Gia_ObjId(p,pObj)-1);
	    Gia_ManForEachAnd(p, pObj, i){
		    if ( Vec_IntFind(vXors_vAdds_share, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,1,1,0], ", Gia_ObjId(p,pObj)-1); //xor and maj 
		    else if ( Vec_IntFind(vAdds_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,1,0,0], ", Gia_ObjId(p,pObj)-1); // maj
		    else if ( Vec_IntFind(vXors_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,0,1,0], ", Gia_ObjId(p,pObj)-1); // xor
		    else
			    fprintf(f_class, "\"%d\": [0,1,0,0,0], ", Gia_ObjId(p,pObj)-1); // and
	    } 
	    Gia_ManForEachCo( p, pObj, i )
		    if(  i < Gia_ManCoNum(p) - 1)
			    fprintf(f_class, "\"%d\": [1,0,0,0,0], ", Gia_ObjId(p,pObj)-1);
		    else
			    fprintf(f_class, "\"%d\": [1,0,0,0,0]", Gia_ObjId(p,pObj)-1);

	    fprintf(f_class, "}");
	    fclose(f_class);
	 }
         //csv
	 else {
		 Gia_ManForEachCi( p, pObj, i )
			 fprintf(f_class, "0,0,0,0,1\n");
		 Gia_ManForEachAnd(p, pObj, i){
			 if (isAdds(Gia_ObjId(p,pObj), vAdds) == 0)
				 //if (isAdds(Gia_ObjId(p,pObj), vXors) == 0)
				 fprintf(f_class, "0,0,0,1,0\n"); //xor 
			 else if(isAdds(Gia_ObjId(p,pObj), vAdds) == 1)
				 fprintf(f_class, "0,0,1,0,0\n"); // maj
			 else
				 fprintf(f_class, "0,1,0,0,0\n"); // and
		 } 
		 Gia_ManForEachCo( p, pObj, i )
			 if(  i < Gia_ManCoNum(p) - 1)
				 fprintf(f_class, "1,0,0,0,0\n");
			 else
				 fprintf(f_class, "1,0,0,0,0\n");

		 fclose(f_class);
	 }


    }


}



// Multiple label(class) on same nodes if possible 

void Gia_EdgelistMultiLabel_issue( Gia_Man_t * p , char *f0, char *f1, char *f2, int multihot)
{
    extern void Acec_TreeVerifyConnections( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Wec_t * vBoxes );

    abctime clk = Abc_Clock();
    Acec_Box_t * pBox = NULL;
    Vec_Int_t * vXors, * vAdds = Ree_ManComputeCuts( p, &vXors, 0 );
    Vec_Int_t * vTemp, * vXorRoots = Acec_FindXorRoots( p, vXors ); 
    Vec_Int_t * vRanks = Acec_RankTrees( p, vXors, vXorRoots ); 
    Vec_Wec_t * vXorLeaves, * vAddBoxes = NULL; 

    Vec_Int_t * vAdds_copy = Vec_IntDup(vAdds);
    Vec_Int_t * vXors_copy = Vec_IntDup(vXors);
    Vec_IntSort(vAdds, 0);  // sorting in ascending order for common search
    Vec_IntSort(vXors, 0); // sorting in ascending order for common search

    Vec_Int_t * vXors_vAdds_share = Vec_IntAlloc( Gia_ManObjNum(p) ); // initialize with upper bound node count
    int common = Vec_IntTwoFindCommon( vXors, vAdds, vXors_vAdds_share);
    if (common > 0){
        //printf("common > 0");
        common = Vec_IntUniqify(vXors_vAdds_share);
    }

    //Vec_IntUniqify(vXors_copy);
    //Vec_IntUniqify(vAdds_copy);
    /*
    printf( "XOR after Cut compute: \n" );
    Vec_IntPrint( vXors_copy );
    printf( "vAdds after Cut compute: \n" );
    Vec_IntPrint( vAdds_copy );
    printf( "XOR roots after reordering: \n" );
    Vec_IntPrint( vXorRoots );
    printf( "xor add share: %d \n" , common );
    if (common > 0){
        Vec_IntPrint( vXors_vAdds_share );
    }
    */
    Gia_ManLevelNum(p);
    Gia_edgelist(p,f0,f1,f2);
    FILE * f_class;
    f_class = fopen (f1, "w"); 
    int i;
    fprintf(f_class, "{");
    Gia_Obj_t * pObj;

    if(multihot!=1){
	    Gia_ManForEachCi( p, pObj, i )
		    fprintf(f_class, "\"%d\": [0,0,0,0,0,1,0,0], ", Gia_ObjId(p,pObj)-1);
	    Gia_ManForEachAnd(p, pObj, i){
		    if ( Vec_IntFind(vXors_vAdds_share, Gia_ObjId(p,pObj)) != -1 ) // xor/maj shared
                if (isAdds(Gia_ObjId(p,pObj), vAdds) == 0)
			        fprintf(f_class, "\"%d\": [0,0,1,0,0,0,1,0], ", Gia_ObjId(p,pObj)-1); //shared & xor root
                else if(isAdds(Gia_ObjId(p,pObj), vAdds) == 1)
			        fprintf(f_class, "\"%d\": [0,0,1,0,0,0,0,1], ", Gia_ObjId(p,pObj)-1); //shared & major root
                else
			        fprintf(f_class, "\"%d\": [0,0,1,0,0,0,0,0], ", Gia_ObjId(p,pObj)-1); //shared & not root
		    else if ( Vec_IntFind(vAdds_copy, Gia_ObjId(p,pObj)) != -1 )
                if (isAdds(Gia_ObjId(p,pObj), vAdds) == 1)
			        fprintf(f_class, "\"%d\": [0,0,0,1,0,0,0,1], ", Gia_ObjId(p,pObj)-1); // maj only & maj root
                else
			        fprintf(f_class, "\"%d\": [0,0,0,1,0,0,0,0], ", Gia_ObjId(p,pObj)-1); // maj only & not maj root
		    else if ( Vec_IntFind(vXors_copy, Gia_ObjId(p,pObj)) != -1 )
                if (isAdds(Gia_ObjId(p,pObj), vAdds) == 0)
			        fprintf(f_class, "\"%d\": [0,0,0,0,1,0,1,0], ", Gia_ObjId(p,pObj)-1); // xor only * xor root
                else
			        fprintf(f_class, "\"%d\": [0,0,0,0,1,0,0,0], ", Gia_ObjId(p,pObj)-1); // xor only & not xor root
		    else
			    fprintf(f_class, "\"%d\": [0,1,0,0,0,0,0,0], ", Gia_ObjId(p,pObj)-1); // and
	    } 
	    Gia_ManForEachCo( p, pObj, i )
		    if(  i < Gia_ManCoNum(p) - 1)
			    fprintf(f_class, "\"%d\": [1,0,0,0,0,0,0,0], ", Gia_ObjId(p,pObj)-1);
		    else
			    fprintf(f_class, "\"%d\": [1,0,0,0,0,0,0,0]", Gia_ObjId(p,pObj)-1);

	    fprintf(f_class, "}");
	    fclose(f_class);
    }
    else {
	    Gia_ManForEachCi( p, pObj, i )
		    fprintf(f_class, "\"%d\": [0,0,0,0,1], ", Gia_ObjId(p,pObj)-1);
	    Gia_ManForEachAnd(p, pObj, i){
		    if ( Vec_IntFind(vXors_vAdds_share, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,1,1,0], ", Gia_ObjId(p,pObj)-1); //xor and maj 
		    else if ( Vec_IntFind(vAdds_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,1,0,0], ", Gia_ObjId(p,pObj)-1); // maj
		    else if ( Vec_IntFind(vXors_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,0,1,0], ", Gia_ObjId(p,pObj)-1); // xor
		    else
			    fprintf(f_class, "\"%d\": [0,1,0,0,0], ", Gia_ObjId(p,pObj)-1); // and
	    } 
	    Gia_ManForEachCo( p, pObj, i )
		    if(  i < Gia_ManCoNum(p) - 1)
			    fprintf(f_class, "\"%d\": [1,0,0,0,0], ", Gia_ObjId(p,pObj)-1);
		    else
			    fprintf(f_class, "\"%d\": [1,0,0,0,0]", Gia_ObjId(p,pObj)-1);

	    fprintf(f_class, "}");
	    fclose(f_class);
    }


}




// Multiple label(class) on same nodes if possible 
void Gia_EdgelistMultiLabelOld( Gia_Man_t * p , char *f0, char *f1, char *f2, int multihot)
{
    extern void Acec_TreeVerifyConnections( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Wec_t * vBoxes );

    abctime clk = Abc_Clock();
    Acec_Box_t * pBox = NULL;
    Vec_Int_t * vXors, * vAdds = Ree_ManComputeCuts( p, &vXors, 0 );
    Vec_Int_t * vTemp, * vXorRoots = Acec_FindXorRoots( p, vXors ); 
    Vec_Int_t * vRanks = Acec_RankTrees( p, vXors, vXorRoots ); 
    Vec_Wec_t * vXorLeaves, * vAddBoxes = NULL; 

    Vec_Int_t * vAdds_copy = Vec_IntDup(vAdds);
    Vec_Int_t * vXors_copy = Vec_IntDup(vXors);
    Vec_IntSort(vAdds, 0);  // sorting in ascending order for common search
    Vec_IntSort(vXors, 0); // sorting in ascending order for common search

    Vec_Int_t * vXors_vAdds_share = Vec_IntAlloc( Gia_ManObjNum(p) ); // initialize with upper bound node count
    int common = Vec_IntTwoFindCommon( vXors, vAdds, vXors_vAdds_share);
    if (common > 0){
        //printf("common > 0");
        common = Vec_IntUniqify(vXors_vAdds_share);
    }

    //Vec_IntUniqify(vXors_copy);
    //Vec_IntUniqify(vAdds_copy);
    /*
    printf( "XOR after Cut compute: \n" );
    Vec_IntPrint( vXors_copy );
    printf( "vAdds after Cut compute: \n" );
    Vec_IntPrint( vAdds_copy );
    printf( "XOR roots after reordering: \n" );
    Vec_IntPrint( vXorRoots );
    printf( "xor add share: %d \n" , common );
    if (common > 0){
        Vec_IntPrint( vXors_vAdds_share );
    }
*/
    Gia_ManLevelNum(p);
    Gia_edgelist(p,f0,f1,f2);
    FILE * f_class;
    f_class = fopen (f1, "w"); 
    int i;
    fprintf(f_class, "{");
    Gia_Obj_t * pObj;
    if(multihot!=1){
	    Gia_ManForEachCi( p, pObj, i )
		    fprintf(f_class, "\"%d\": [0,0,0,0,0,1], ", Gia_ObjId(p,pObj)-1);
	    Gia_ManForEachAnd(p, pObj, i){
		    if ( Vec_IntFind(vXors_vAdds_share, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,1,0,0,0], ", Gia_ObjId(p,pObj)-1); //xor and maj 
		    else if ( Vec_IntFind(vAdds_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,0,1,0,0], ", Gia_ObjId(p,pObj)-1); // maj
		    else if ( Vec_IntFind(vXors_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,0,0,1,0], ", Gia_ObjId(p,pObj)-1); // xor
		    else
			    fprintf(f_class, "\"%d\": [0,1,0,0,0,0], ", Gia_ObjId(p,pObj)-1); // and
	    } 
	    Gia_ManForEachCo( p, pObj, i )
		    if(  i < Gia_ManCoNum(p) - 1)
			    fprintf(f_class, "\"%d\": [1,0,0,0,0,0], ", Gia_ObjId(p,pObj)-1);
		    else
			    fprintf(f_class, "\"%d\": [1,0,0,0,0,0]", Gia_ObjId(p,pObj)-1);

	    fprintf(f_class, "}");
	    fclose(f_class);
    }
    else {
	    Gia_ManForEachCi( p, pObj, i )
		    fprintf(f_class, "\"%d\": [0,0,0,0,1], ", Gia_ObjId(p,pObj)-1);
	    Gia_ManForEachAnd(p, pObj, i){
		    if ( Vec_IntFind(vXors_vAdds_share, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,1,1,0], ", Gia_ObjId(p,pObj)-1); //xor and maj 
		    else if ( Vec_IntFind(vAdds_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,1,0,0], ", Gia_ObjId(p,pObj)-1); // maj
		    else if ( Vec_IntFind(vXors_copy, Gia_ObjId(p,pObj)) != -1 )
			    fprintf(f_class, "\"%d\": [0,0,0,1,0], ", Gia_ObjId(p,pObj)-1); // xor
		    else
			    fprintf(f_class, "\"%d\": [0,1,0,0,0], ", Gia_ObjId(p,pObj)-1); // and
	    } 
	    Gia_ManForEachCo( p, pObj, i )
		    if(  i < Gia_ManCoNum(p) - 1)
			    fprintf(f_class, "\"%d\": [1,0,0,0,0], ", Gia_ObjId(p,pObj)-1);
		    else
			    fprintf(f_class, "\"%d\": [1,0,0,0,0]", Gia_ObjId(p,pObj)-1);

	    fprintf(f_class, "}");
	    fclose(f_class);
    }


}

//single label
void Gia_EdgelistGraphSAGE( Gia_Man_t * p , char *f0, char *f1, char *f2)
{
    extern void Acec_TreeVerifyConnections( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Wec_t * vBoxes );

    abctime clk = Abc_Clock();
    Acec_Box_t * pBox = NULL;
    Vec_Int_t * vXors, * vAdds = Ree_ManComputeCuts( p, &vXors, 0 );
    Vec_Int_t * vTemp, * vXorRoots = Acec_FindXorRoots( p, vXors ); 
    Vec_Int_t * vRanks = Acec_RankTrees( p, vXors, vXorRoots ); 
    Vec_Wec_t * vXorLeaves, * vAddBoxes = NULL; 
    /*
    Vec_Int_t * vAdds_copy = Vec_IntDup(vAdds);
    Vec_Int_t * vXors_copy = Vec_IntDup(vXors);
    Vec_IntSort(vAdds, 0);  // sorting in ascending order for common search
    Vec_IntSort(vXors, 0); // sorting in ascending order for common search
    Vec_Int_t * vXors_vAdds_share = Vec_IntAlloc( Gia_ManObjNum(p) ); // initialize with upper bound node count
    int common = Vec_IntTwoFindCommon( vXors, vAdds, vXors_vAdds_share);
    if (common > 0){
        printf("common > 0");
        common = Vec_IntUniqify(vXors_vAdds_share);
    }

    Vec_IntUniqify(vXors_copy);
    Vec_IntUniqify(vAdds_copy);

    printf( "XOR after Cut compute: \n" );
    Vec_IntPrint( vXors_copy );
    printf( "vAdds after Cut compute: \n" );
    Vec_IntPrint( vAdds_copy );
    printf( "XOR roots after reordering: \n" );
    Vec_IntPrint( vXorRoots );
    printf( "xor add share: %d \n" , common );
    if (common > 0){
        Vec_IntPrint( vXors_vAdds_share );
    }
    */

    Gia_ManLevelNum(p);
    Gia_edgelist(p,f0,f1,f2);
    FILE * f_class;
    f_class = fopen (f1, "w"); 
    int i;

if JSON {
    fprintf(f_class, "{");
    Gia_Obj_t * pObj;
    Gia_ManForEachCi( p, pObj, i )
        fprintf(f_class, "\"%d\": [0,0,0,0,1], ", Gia_ObjId(p,pObj)-1);
    Gia_ManForEachAnd(p, pObj, i){
        if (isAdds(Gia_ObjId(p,pObj), vAdds) == 0)
        //if (isAdds(Gia_ObjId(p,pObj), vXors) == 0)
            fprintf(f_class, "\"%d\": [0,0,0,1,0], ", Gia_ObjId(p,pObj)-1); //xor 
        else if(isAdds(Gia_ObjId(p,pObj), vAdds) == 1)
            fprintf(f_class, "\"%d\": [0,0,1,0,0], ", Gia_ObjId(p,pObj)-1); // maj
        else
            fprintf(f_class, "\"%d\": [0,1,0,0,0], ", Gia_ObjId(p,pObj)-1); // and
    } 
    Gia_ManForEachCo( p, pObj, i )
        if(  i < Gia_ManCoNum(p) - 1)
            fprintf(f_class, "\"%d\": [1,0,0,0,0], ", Gia_ObjId(p,pObj)-1);
        else
            fprintf(f_class, "\"%d\": [1,0,0,0,0]", Gia_ObjId(p,pObj)-1);

    fprintf(f_class, "}");
    fclose(f_class);
}
else {
    Gia_Obj_t * pObj;
    Gia_ManForEachCi( p, pObj, i )
        fprintf(f_class, "0,0,0,0,1\n");
    Gia_ManForEachAnd(p, pObj, i){
        if (isAdds(Gia_ObjId(p,pObj), vAdds) == 0)
        //if (isAdds(Gia_ObjId(p,pObj), vXors) == 0)
            fprintf(f_class, "0,0,0,1,0\n"); //xor 
        else if(isAdds(Gia_ObjId(p,pObj), vAdds) == 1)
            fprintf(f_class, "0,0,1,0,0\n"); // maj
        else
            fprintf(f_class, "0,1,0,0,0\n"); // and
    } 
    Gia_ManForEachCo( p, pObj, i )
        if(  i < Gia_ManCoNum(p) - 1)
            fprintf(f_class, "1,0,0,0,0\n");
        else
            fprintf(f_class, "1,0,0,0,0\n");

    fclose(f_class);
}

 
}
Acec_Box_t * Acec_ProduceBox( Gia_Man_t * p, int fVerbose )
{
    extern void Acec_TreeVerifyConnections( Gia_Man_t * p, Vec_Int_t * vAdds, Vec_Wec_t * vBoxes );

    abctime clk = Abc_Clock();
    Acec_Box_t * pBox = NULL;
    Vec_Int_t * vXors, * vAdds = Ree_ManComputeCuts( p, &vXors, 0 );
    Vec_Int_t * vTemp, * vXorRoots = Acec_FindXorRoots( p, vXors ); 
    Vec_Int_t * vRanks = Acec_RankTrees( p, vXors, vXorRoots ); 
    Vec_Wec_t * vXorLeaves, * vAddBoxes = NULL; 

    Gia_ManLevelNum(p);

    //Acec_CheckXors( p, vXors );

    Ree_ManPrintAdders( vAdds, 1 );
    if ( fVerbose )
        printf( "Detected %d full-adders and %d half-adders.  Found %d XOR-cuts.  ", Ree_ManCountFadds(vAdds), Vec_IntSize(vAdds)/6-Ree_ManCountFadds(vAdds), Vec_IntSize(vXors)/4 );
    if ( fVerbose )
        Abc_PrintTime( 1, "Time", Abc_Clock() - clk );

    vXorRoots = Acec_OrderTreeRoots( p, vAdds, vTemp = vXorRoots, vRanks );
    Vec_IntFree( vTemp );
    Vec_IntFree( vRanks );

    vRanks = Acec_RankTrees( p, vXors, vXorRoots ); 
    vXorLeaves = Acec_FindXorLeaves( p, vXors, vAdds, vXorRoots, vRanks, &vAddBoxes ); 
    Vec_IntFree( vRanks );
    int Entry=0; int i=0;
    //printf( "XOR roots after reordering: \n" );
    //Vec_IntPrint( vXorRoots );
    //printf( "XOR leaves: \n" );
    //Vec_WecPrint( vXorLeaves, 0 );
    //printf( "Adder boxes: \n" );
    //Vec_WecPrint( vAddBoxes, 0 );

    Acec_TreeVerifyConnections( p, vAdds, vAddBoxes );

    pBox = Acec_FindBox( p, vAdds, vAddBoxes, vXorLeaves, vXorRoots );
    //Vec_WecFree( vAddBoxes );

    if ( fVerbose )
        Acec_TreePrintBox( pBox, vAdds );

    Vec_IntFree( vXorRoots );
    Vec_WecFree( vXorLeaves );

    Vec_IntFree( vXors );
    Vec_IntFree( vAdds );

    return pBox;
}


////////////////////////////////////////////////////////////////////////
///                       END OF FILE                                ///
////////////////////////////////////////////////////////////////////////


ABC_NAMESPACE_IMPL_END

