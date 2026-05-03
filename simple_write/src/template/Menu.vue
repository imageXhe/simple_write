<template >
  <div >
    <a-layout style="background: #fff">

      <a-layout  :class="expand_state ? 'expand_enter' : 'expand_leave'">
        
          <a-layout-header class="layout_header" v-if="expand_state">
            <div style="padding-top:5px">
              <a-flex justify="space-evenly" >
                <a-tooltip :title="fileListTitle" :arrow="false" placement="bottom">
                  <a-button ghost size="large"
                  :class="ButtonStyle.FileListButton ? 'button_selected' : 'button_selected_not'" 
                  @click="FileListButton">
                    <FolderOpenOutlined />
                  </a-button> 
                </a-tooltip>
                <a-tooltip :title="searchTitle" :arrow="false" placement="bottom">
                  <a-button ghost size="large"
                  :class="ButtonStyle.SearchButton ? 'button_selected' : 'button_selected_not'" 
                  @click="SearchButton">
                    <SearchOutlined />
                  </a-button> 
                </a-tooltip>
                <a-tooltip :title="bookmarkTitle" :arrow="false" placement="bottom">
                  <a-button ghost size="large"
                  :class="ButtonStyle.BookmarkButton ? 'button_selected' : 'button_selected_not'"
                  @click="BookmarkButton">
                    <BookOutlined />
                  </a-button> 
                </a-tooltip>

              </a-flex> 
            </div>    
          </a-layout-header>
          
          <a-layout-content v-if="expand_state" class="layout_content"> 
            <FileList />
          </a-layout-content>

          <!-- <a-layout-footer class="layout_footer" v-if="expand_state">
              <Setting /> 
          </a-layout-footer> -->

      </a-layout>
    </a-layout>
  </div>  

</template>


<script setup>
import { ref, onMounted, computed } from 'vue'
import {
  FolderOpenOutlined,
  SearchOutlined,
  BookOutlined,
} from '@ant-design/icons-vue'
import FileList from '../menu/FileList.vue'
// import Setting from '../menu/Setting.vue'
import { useI18n, loadLanguage } from "../locales";

const { t } = useI18n();

const fileListTitle = computed(() => t('file.fileList'));
const searchTitle = computed(() => t('file.search'));
const bookmarkTitle = computed(() => t('file.bookmark'));

onMounted(async () => {
  await loadLanguage();
});

const expand_state = ref(true)

const ButtonStyle = ref({
  FileListButton: true,
  SearchButton: false,
  BookmarkButton: false,
})
const FileListButton = () => {
  if (!ButtonStyle.value.FileListButton) {
    ButtonStyle.value.FileListButton = true
    ButtonStyle.value.SearchButton = false
    ButtonStyle.value.BookmarkButton = false
  } 
}
const SearchButton = () => {
  if (!ButtonStyle.value.SearchButton) {
    ButtonStyle.value.FileListButton = false
    ButtonStyle.value.SearchButton = true
    ButtonStyle.value.BookmarkButton = false
  } 
}
const BookmarkButton = () => {
  if (!ButtonStyle.value.BookmarkButton) {
    ButtonStyle.value.FileListButton = false
    ButtonStyle.value.SearchButton = false
    ButtonStyle.value.BookmarkButton = true
  } 
}

</script>

<style scoped>
.layout_header {
  background-color: #fff;
  border-bottom: 1px solid #e8e8e8;
}
.layout_content {
  border-top: 1px solid brown;
  height: calc(100vh - 50px - 50px);
  background: #fff;
  padding: 0;
}
.button_selected {
  background-color: #b3d8fb;
  color: black;
}
.button_selected_not {
  color: white;
}

.expand_enter {
  width: 225px;
  height: 100vh;
  display: block;
  animation: ExpandEnter 0.1s;
  background-color: #fff;
}
.expand_leave {
  height: 100vh;
  display: none;
  animation: ExpandLeave 0.1s ;
  background-color: #fff;
}
@keyframes ExpandEnter {
  0% {
    width: 0;
  }
  100% {
    width: 225px;
  }
}
@keyframes ExpandLeave {
  0% {
    width: 225px;
    display: block;
  }
  100% {
    width: 0;
  }
}
</style>
