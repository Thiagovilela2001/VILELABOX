#pragma once

#include <QVector3D>
#include <QVector>

namespace allshader {
class RGBData;
}

class allshader::RGBData {
    QVector<QVector3D> mData;

  public:
    void addForRectangle(float r, float g, float b) {
        mData.push_back({r, g, b});
        mData.push_back({r, g, b});
        mData.push_back({r, g, b});
        mData.push_back({r, g, b});
        mData.push_back({r, g, b});
        mData.push_back({r, g, b});
    }
    /// Colours for addRectangle() with a vertical gradient: `top` at y1,
    /// `bottom` at y2 (same vertex order as VertexData::addRectangle).
    void addForGradientRectangle(const QVector3D& top, const QVector3D& bottom) {
        mData.push_back(top);
        mData.push_back(top);
        mData.push_back(bottom);
        mData.push_back(bottom);
        mData.push_back(bottom);
        mData.push_back(top);
    }
    void clear() {
        mData.clear();
    }
    void reserve(int size) {
        mData.reserve(size);
    }
    int size() const {
        return mData.size();
    }
    const QVector3D* constData() const {
        return mData.constData();
    }
};
