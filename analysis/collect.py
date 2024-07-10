import matplotlib.pyplot as plt
import numpy as np

# 数据
names = ['DAC23[]', 'ASILOMAR23[]', 'ASPLOS24[]', 'DAC24[]', 'EsynTurbo']
delay = [285, 93.6, 1550, 2028, 394483.06]
area = [102, 8, 132, 2360, 219638.48]

# 归一化处理
delay_normalized = (delay - np.min(delay)) / (np.max(delay) - np.min(delay))
area_normalized = (area - np.min(area)) / (np.max(area) - np.min(area))

# 设置图形大小
plt.figure(figsize=(10, 6))

# 设置柱状图位置
bar_positions = np.arange(len(names))

# 调整颜色的亮度或饱和度
color_delay = 'lightskyblue'
color_area = 'lightcoral'

# 绘制delay柱状图
plt.bar(bar_positions, delay_normalized, color=color_delay, width=0.4, label='Delay')

# 绘制area柱状图
plt.bar(bar_positions + 0.4, area_normalized, color=color_area, width=0.4, label='Area')

# 设置纵轴标签和标题
plt.ylabel('Normalized value', fontsize=12)
plt.title('Normalized Delay and Area Comparison of the Maximum Circuit', fontsize=14)

# 设置横轴刻度标签
plt.xticks(bar_positions + 0.2, names, rotation='horizontal', fontsize=10)

# 添加图例
plt.legend()

# 调整坐标轴标签字体大小
plt.tick_params(axis='both', which='major', labelsize=10)

# 保存图像为SVG格式
# 保存图像为PDF格式
plt.savefig('scale.pdf', format='pdf')